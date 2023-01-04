use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let api_json_file = format!("{manifest_dir}/godot_headers/gdnative_api.json");
    let out_dir = env::var("OUT_DIR").unwrap();

    header_binding::generate(&manifest_dir, &out_dir);

    api_wrapper::generate(&api_json_file, &out_dir, "api_wrapper.rs");

    // Only re-run build.rs if the gdnative_api.json file has been updated.
    // Manually rebuilding the crate will ignore this.
    println!("cargo:rerun-if-changed={api_json_file}");
}

mod header_binding {
    use std::path::{Path, PathBuf};

    fn apple_include_path() -> Result<String, std::io::Error> {
        use std::process::Command;

        let target = std::env::var("TARGET").unwrap();
        let platform = if target.contains("apple-darwin") {
            "macosx"
        } else if target == "x86_64-apple-ios" || target == "aarch64-apple-ios-sim" {
            "iphonesimulator"
        } else if target == "aarch64-apple-ios" {
            "iphoneos"
        } else {
            panic!("not building for macOS or iOS");
        };

        // run `xcrun --sdk iphoneos --show-sdk-path`
        let output = Command::new("xcrun")
            .args(["--sdk", platform, "--show-sdk-path"])
            .output()?
            .stdout;
        let prefix = std::str::from_utf8(&output)
            .expect("invalid output from `xcrun`")
            .trim_end();

        let suffix = "usr/include";
        let directory = format!("{prefix}/{suffix}");

        Ok(directory)
    }

    fn add_android_include_paths(mut builder: bindgen::Builder) -> bindgen::Builder {
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        let target_triple = std::env::var("TARGET").unwrap();

        assert_eq!("android", &target_os);

        let android_sdk_root =
            std::env::var("ANDROID_SDK_ROOT").expect("ANDROID_SDK_ROOT must be set");
        let android_sdk_root = Path::new(&android_sdk_root).to_path_buf();

        // Note: cfg!(target_os) and cfg!(target_arch) refer to the target of the build script:
        // in other words, the host machine instead of the target of gdnative-sys. They are confusing
        // and have been erroneously used for target platforms in this library in the past. Make sure
        // to double-check them wherever they occur.

        assert!(
            cfg!(target_os = "macos") || // All macOS architectures are supported
            cfg!(target_arch = "x86_64"),
            "unsupported host architecture: build from x86_64 instead"
        );

        let mut android_ndk_root: Option<PathBuf> = None;

        let android_ndk_folder = Path::join(&android_sdk_root, "ndk");
        if android_ndk_folder.exists() {
            // New NDK
            let available_ndk_versions: Vec<_> = std::fs::read_dir(android_ndk_folder.clone())
                .unwrap()
                .map(|dir| dir.unwrap().path())
                .collect();

            if !available_ndk_versions.is_empty() {
                let ndk_version = std::env::var("ANDROID_NDK_VERSION");

                if let Ok(ndk_version) = ndk_version {
                    if available_ndk_versions
                        .iter()
                        .map(|p| p.file_name())
                        .any(|p| {
                            p.is_some() && p.unwrap().to_string_lossy().eq(ndk_version.as_str())
                        })
                    {
                        // Asked version is available
                        android_ndk_root = Some(Path::join(&android_ndk_folder, ndk_version))
                    } else {
                        panic!(
                            "no available android ndk versions matches {ndk_version}. Available versions: {available_ndk_versions:?}"
                        )
                    }
                } else {
                    // No NDK version chosen, chose the most recent one and issue a warning
                    println!("cargo:warning=Multiple android ndk versions have been detected.");
                    println!("cargo:warning=You should chose one using ANDROID_NDK_VERSION environment variable to have reproducible builds.");
                    println!("cargo:warning=Available versions: {available_ndk_versions:?}");

                    let ndk_version = available_ndk_versions
                        .iter()
                        .filter_map(|p| p.file_name())
                        .filter_map(|v| semver::Version::parse(v.to_string_lossy().as_ref()).ok())
                        .max()
                        .unwrap();

                    println!("cargo:warning=Automatically chosen version: {ndk_version} (latest)");

                    android_ndk_root =
                        Some(Path::join(&android_ndk_folder, ndk_version.to_string()));
                }
            }
        }

        let android_ndk_bundle_folder = Path::join(&android_sdk_root, "ndk-bundle");
        if android_ndk_root.is_none() && android_ndk_bundle_folder.exists() {
            // Old NDK
            android_ndk_root = Some(android_ndk_bundle_folder);
        }

        let android_ndk_root = android_ndk_root.expect("Android ndk needs to be installed");

        builder = builder
            .clang_arg("-I")
            .clang_arg(Path::join(&android_ndk_root, "sysroot/usr/include").to_string_lossy());
        builder = builder.clang_arg("-I").clang_arg(
            Path::join(&android_ndk_root, "sources/cxx-stl/llvm-libc++/include").to_string_lossy(),
        );
        builder = builder.clang_arg("-I").clang_arg(
            Path::join(&android_ndk_root, "sources/cxx-stl/llvm-libc++abi/include")
                .to_string_lossy(),
        );
        builder = builder.clang_arg("-I").clang_arg(
            Path::join(&android_ndk_root, "sources/android/support/include").to_string_lossy(),
        );

        let host_tag = {
            if cfg!(target_os = "windows") {
                "windows-x86_64"
            } else if cfg!(target_os = "macos") {
                "darwin-x86_64"
            } else if cfg!(target_os = "linux") {
                "linux-x86_64"
            } else {
                panic!("unsupported host OS: build from Windows, MacOS, or Linux instead");
            }
        };

        builder = builder.clang_arg("-I").clang_arg(
            Path::join(
                &android_ndk_root,
                format!("toolchains/llvm/prebuilt/{}/sysroot/usr/include", &host_tag),
            )
            .to_string_lossy(),
        );

        builder = builder.clang_arg("-I").clang_arg(
            Path::join(
                &android_ndk_root,
                format!(
                    "toolchains/llvm/prebuilt/{host}/sysroot/usr/include/{target_triple}",
                    host = &host_tag,
                    target_triple = &target_triple,
                ),
            )
            .to_string_lossy(),
        );

        builder
    }

    #[allow(clippy::single_match)]
    pub(crate) fn generate(manifest_dir: &str, out_dir: &str) {
        // on mac/iOS this will be modified, so it is marked as mutable.
        // on all other targets, this `mut` will be unused and the complainer compiles.t s
        #[allow(unused_mut)]
        let mut builder = bindgen::Builder::default()
            .header("godot_headers/gdnative_api_struct.gen.h")
            .allowlist_type("godot.*")
            .allowlist_function("godot.*")
            .allowlist_var("godot.*")
            .allowlist_type("GDNATIVE.*")
            .derive_default(true)
            .ignore_functions()
            .size_t_is_usize(true)
            .ctypes_prefix("libc")
            .clang_arg(format!("-I{manifest_dir}/godot_headers"));

        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let target_vendor = std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap();
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

        if target_vendor == "apple" {
            match apple_include_path() {
                Ok(osx_include_path) => {
                    builder = builder.clang_arg("-I").clang_arg(osx_include_path);
                }
                _ => {}
            }
        }

        // Workaround for https://github.com/rust-lang/rust-bindgen/issues/1211: manually set
        // target triple to `arm64-apple-ios` in place of `aarch64-apple-ios`.
        if target_arch == "aarch64" && target_os == "ios" {
            if target_env == "sim" {
                builder = builder.clang_arg("--target=arm64-apple-ios-sim");
            } else {
                builder = builder.clang_arg("--target=arm64-apple-ios");
            }
        }

        // Workaround: Microsoft extensions aren't enabled by default for the `gnu` toolchain
        // on Windows. We need to enable it manually, or MSVC headers will fail to parse. We
        // also need to manually define architecture macros, or the build will fail with an
        // "Unsupported architecture" error.
        //
        // This does not happen when the `msvc` toolchain is used.
        if target_os == "windows" && target_env == "gnu" {
            let arch_macro = match target_arch.as_str() {
                "x86" => "_M_IX86",
                "x86_64" => "_M_X64",
                "arm" => "_M_ARM",
                "aarch64" => "_M_ARM64",
                _ => panic!("architecture {target_arch} not supported on Windows"),
            };

            builder = builder
                .clang_arg("-fms-extensions")
                .clang_arg("-fmsc-version=1300")
                .clang_arg(format!("-D{arch_macro}=100"));
        }

        if target_os == "android" {
            builder = add_android_include_paths(builder);
        }

        let bindings = builder.generate().expect("Unable to generate bindings");

        let out_path = PathBuf::from(out_dir);
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}

mod api_wrapper {
    use proc_macro2::{Ident, TokenStream};
    use quote::{format_ident, quote, ToTokens};
    use std::convert::AsRef;
    use std::fs::File;
    use std::io::Write as _;
    use std::path;

    use miniserde::{de, Deserialize};
    miniserde::make_place!(Place);

    #[derive(Debug, Deserialize)]
    struct ApiRoot {
        core: Api,
        extensions: Vec<Api>,
    }

    #[derive(Debug, Deserialize)]
    struct Api {
        //name: Option<String>, // currently not used
        #[serde(rename = "type")]
        type_: String,
        version: Version,
        next: Option<Box<Api>>,
        #[serde(rename = "api")]
        functions: Vec<Function>,
    }

    #[derive(Debug, Deserialize)]
    struct Version {
        major: u32,
        minor: u32,
    }

    #[derive(Debug, Deserialize)]
    struct Function {
        name: String,
        return_type: String,
        arguments: Vec<Argument>,
    }

    #[derive(Debug)]
    struct Argument {
        type_: String,
        name: String,
    }

    impl ApiRoot {
        fn all_apis(&self) -> Vec<&Api> {
            let mut result = Vec::new();
            result.extend(self.core.nexts());
            for extension in &self.extensions {
                // Workaround: Interface for the ANDROID extension is broken in Godot 3.2,
                // making it impossible to support the extension in a forward compatible manner.
                // The extension is disabled until the compatibility policy is figured out in
                // upstream.
                //
                // Related issue: #296
                if extension.type_ == "ANDROID" {
                    continue;
                }

                result.extend(extension.nexts());
            }
            result
        }
    }

    impl Api {
        fn nexts(&self) -> Vec<&Api> {
            let mut api = self;
            let mut result = vec![api];
            while let Some(next) = api.next.as_ref() {
                result.push(next);
                api = next;
            }
            result
        }

        fn macro_ident(&self) -> Ident {
            format_ident!(
                "{}_{}_{}",
                self.type_.to_lowercase(),
                self.version.major,
                self.version.minor
            )
        }

        fn godot_api_type(&self) -> Ident {
            godot_api_type_ident(&self.type_)
        }

        fn godot_api_struct(&self) -> Ident {
            godot_api_struct_ident(&self.type_, self.version.major, self.version.minor)
        }
    }

    impl Function {
        fn rust_name(&self) -> Ident {
            format_ident!("{}", self.name)
        }

        fn rust_args(&self) -> TokenStream {
            let arg = &self.arguments;
            quote!(#(#arg),*)
        }

        fn rust_return_type(&self) -> TokenStream {
            c_type_to_rust_type(&self.return_type)
        }
    }

    impl ToTokens for Function {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let args = self.rust_args();
            let return_type = self.rust_return_type();
            tokens.extend(quote!(unsafe extern "C" fn(#args) -> #return_type));
        }
    }

    impl Argument {
        fn rust_name(&self) -> Ident {
            match self.name.trim_start_matches("p_") {
                "self" => format_ident!("{}", "self_"),
                arg_name => format_ident!("{}", arg_name),
            }
        }

        fn rust_type(&self) -> TokenStream {
            c_type_to_rust_type(&self.type_)
        }
    }

    impl ToTokens for Argument {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let name = self.rust_name();
            let type_ = self.rust_type();
            tokens.extend(quote!(#name: #type_));
        }
    }

    // Used to convert [String, String] in JSON into the Argument struct.
    impl Deserialize for Argument {
        fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor {
            impl de::Visitor for Place<Argument> {
                fn seq(&mut self) -> miniserde::Result<Box<dyn de::Seq + '_>> {
                    Ok(Box::new(ArgumentBuilder {
                        out: &mut self.out,
                        tuple: (None, None),
                    }))
                }
            }

            struct ArgumentBuilder<'a> {
                out: &'a mut Option<Argument>,
                tuple: (Option<String>, Option<String>),
            }

            impl<'a> de::Seq for ArgumentBuilder<'a> {
                fn element(&mut self) -> miniserde::Result<&mut dyn de::Visitor> {
                    if self.tuple.0.is_none() {
                        Ok(Deserialize::begin(&mut self.tuple.0))
                    } else if self.tuple.1.is_none() {
                        Ok(Deserialize::begin(&mut self.tuple.1))
                    } else {
                        Err(miniserde::Error)
                    }
                }

                fn finish(&mut self) -> miniserde::Result<()> {
                    if let (Some(a), Some(b)) = (self.tuple.0.take(), self.tuple.1.take()) {
                        *self.out = Some(Argument { type_: a, name: b });
                        Ok(())
                    } else {
                        Err(miniserde::Error)
                    }
                }
            }

            Place::new(out)
        }
    }

    fn godot_api_struct_ident(type_: &str, version_major: u32, version_minor: u32) -> Ident {
        match (type_, version_major, version_minor) {
            ("CORE", 1, 0) => format_ident!("godot_gdnative_core_api_struct"),
            ("CORE", maj, min) => format_ident!("godot_gdnative_core_{}_{}_api_struct", maj, min),
            ("NATIVESCRIPT", 1, 0) => format_ident!("godot_gdnative_ext_nativescript_api_struct"),
            ("NATIVESCRIPT", maj, min) => {
                format_ident!("godot_gdnative_ext_nativescript_{}_{}_api_struct", maj, min)
            }
            ("PLUGINSCRIPT", 1, 0) => format_ident!("godot_gdnative_ext_pluginscript_api_struct"),
            // The Android 1.0 API in Godot 3.1 was "reversioned" to be the Android 1.1 API in Godot 3.2.
            // Godot 3.2 does not have an Android 1.0 API.
            // Both Android 1.0, and Android 1.1 refer to the same struct in either case.
            ("ANDROID", 1, 0) => format_ident!("godot_gdnative_ext_android_api_struct"),
            ("ANDROID", 1, 1) => format_ident!("godot_gdnative_ext_android_api_struct"),
            ("ARVR", 1, 1) => format_ident!("godot_gdnative_ext_arvr_api_struct"),
            ("VIDEODECODER", 0, 1) => format_ident!("godot_gdnative_ext_videodecoder_api_struct"),
            ("NET", 3, 1) => format_ident!("godot_gdnative_ext_net_api_struct"),
            ("NET", maj, min) => format_ident!("godot_gdnative_ext_net_{}_{}_api_struct", maj, min),
            api => panic!("Unknown API type and version: {api:?}"),
        }
    }

    fn godot_api_type_ident(type_: &str) -> Ident {
        match type_ {
            "CORE" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_CORE"),
            "NATIVESCRIPT" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_EXT_NATIVESCRIPT"),
            "PLUGINSCRIPT" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_EXT_PLUGINSCRIPT"),
            "ANDROID" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_EXT_ANDROID"),
            "ARVR" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_EXT_ARVR"),
            "VIDEODECODER" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_EXT_VIDEODECODER"),
            "NET" => format_ident!("GDNATIVE_API_TYPES_GDNATIVE_EXT_NET"),
            other => panic!("Unknown API type: {other:?}"),
        }
    }

    pub fn generate(
        from_json: &dyn AsRef<path::Path>,
        to: &dyn AsRef<path::Path>,
        file_name: &str,
    ) {
        let from_json = from_json.as_ref();
        let to = to.as_ref();
        let api_json_file = std::fs::read_to_string(from_json)
            .unwrap_or_else(|_| panic!("No such file: {from_json:?}"));
        eprintln!("{}", &api_json_file);
        let api_root: ApiRoot = miniserde::json::from_str(&api_json_file)
            .unwrap_or_else(|_| panic!("Could not parse ({from_json:?}) into ApiRoot"));

        // Note: this code ensured that Godot 4 (which was back then GDNative 1.3) wasn't actually used.
        // Godot uses now GDExtension, so this no longer applies. In fact, different module APIs all have different versions.
        // See also: https://github.com/godot-rust/godot-rust/issues/904

        // Listed versions for Godot 3.5 RC:
        //  * CORE 1.0
        //  * CORE 1.1
        //  * CORE 1.2
        //  * NATIVESCRIPT 1.0
        //  * NATIVESCRIPT 1.1
        //  * PLUGINSCRIPT 1.0
        //  * ARVR 1.1
        //  * VIDEODECODER 0.1
        //  * NET 3.1
        //  * NET 3.2

        let struct_fields = godot_api_functions(&api_root);
        let impl_constructor = api_constructor(&api_root);
        let wrapper = quote! {
            pub struct GodotApi{
                #struct_fields
            }
            impl GodotApi {
                #impl_constructor
            }
        };
        let mut wrapper_file = File::create(to.join(file_name))
            .unwrap_or_else(|_| panic!("Couldn't create output file: {:?}", to.join(file_name)));
        write!(wrapper_file, "{wrapper}").unwrap();
    }

    fn godot_api_functions(api: &ApiRoot) -> TokenStream {
        let mut result = TokenStream::new();
        for api in api.all_apis() {
            for function in &api.functions {
                let function_name = function.rust_name();
                result.extend(quote!(pub #function_name: #function,));
            }
        }
        result
    }

    fn api_constructor(api: &ApiRoot) -> TokenStream {
        let mut godot_apis = TokenStream::new();
        let mut struct_field_bindings = TokenStream::new();
        let mut constructed_struct_fields = TokenStream::new();
        for api in api.all_apis() {
            let i = api.macro_ident();
            let gd_api_type = api.godot_api_type();
            let v_maj = api.version.major;
            let v_min = api.version.minor;
            let gd_api_struct = api.godot_api_struct();
            godot_apis.extend(quote! {
                let #i = find_api_ptr(core_api_struct, #gd_api_type, #v_maj, #v_min)? as *const #gd_api_struct;
            });
            for function in &api.functions {
                let function_name = function.rust_name();
                let message = format!(
                    "API function missing: {}.{}",
                    api.godot_api_struct(),
                    function_name
                );

                // Workaround: rustc has trouble dealing with a large amount of returns within the
                // same expression when optimization is enabled, causing the build to appear to halt.
                // Separating the try expressions into let bindings resolved this problem.
                struct_field_bindings.extend(quote! {
                    let #function_name = map_option_to_init_error((*#i).#function_name, #message)?;
                });
                constructed_struct_fields.extend(quote! {
                    #function_name,
                });
            }
        }
        quote! {
            pub unsafe fn from_raw(core_api_struct: *const godot_gdnative_core_api_struct) -> std::result::Result<Self, InitError> {
                #godot_apis
                #struct_field_bindings
                std::result::Result::Ok(GodotApi{
                    #constructed_struct_fields
                })
            }
        }
    }

    fn parse_c_type(mut c_type: &str) -> (bool, i8, &str) {
        c_type = c_type.trim();
        let is_const = c_type.starts_with("const ");
        if is_const {
            c_type = c_type.trim_start_matches("const ");
        }
        c_type = c_type.trim();
        let mut ptr_count = 0;
        while c_type.ends_with('*') {
            ptr_count += 1;
            c_type = c_type[..c_type.len() - 1].trim();
        }
        (is_const, ptr_count, c_type)
    }

    fn c_type_to_rust_type(c_type: &str) -> TokenStream {
        let (is_const, ptr_count, base_c_type) = parse_c_type(c_type);
        let rust_ptrs = match (ptr_count, is_const) {
            (0, _) => quote!(),
            (1, true) => quote!(*const ),
            (1, false) => quote!(*mut ),
            (2, true) => quote!(*mut *const ),
            _ => panic!("Unknown C type (Too many pointers?): {c_type:?}"),
        };
        let rust_type = match base_c_type {
            "void" => {
                if ptr_count == 0 {
                    quote!(())
                } else {
                    quote!(std::ffi::c_void)
                }
            }
            "bool" => quote!(bool),
            "uint8_t" => quote!(u8),
            "uint32_t" => quote!(u32),
            "uint64_t" => quote!(u64),
            "int64_t" => quote!(i64),
            "int" => quote!(std::os::raw::c_int),
            "double" => quote!(std::os::raw::c_double),
            "char" => quote!(std::os::raw::c_char),
            "signed char" => quote!(std::os::raw::c_schar),
            "size_t" => quote!(usize),
            "JNIEnv" => quote!(std::ffi::c_void),
            "jobject" => quote!(*mut std::ffi::c_void),
            godot_type => {
                let i = format_ident!("{}", godot_type);
                quote!(#i)
            }
        };
        quote!(#rust_ptrs #rust_type)
    }
}

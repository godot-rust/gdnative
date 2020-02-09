use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let api_json_file = format!("{}/godot_headers/gdnative_api.json", manifest_dir);
    let out_dir = env::var("OUT_DIR").unwrap();

    header_binding::generate(&manifest_dir, &out_dir);

    api_wrapper::generate(&api_json_file, &out_dir, "api_wrapper.rs");
}

mod header_binding {
    use std::path::PathBuf;

    fn osx_include_path() -> Result<String, std::io::Error> {
        use std::process::Command;

        let output = Command::new("xcode-select").arg("-p").output()?.stdout;
        let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcode-select`");
        let prefix = prefix_str.trim_end();

        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

        let platform = if target_os == "macos" {
            "MacOSX"
        } else if target_os == "ios" {
            "iPhoneOS"
        } else {
            panic!("not building for macOS or iOS");
        };

        let infix = if prefix == "/Library/Developer/CommandLineTools" {
            format!("SDKs/{}.sdk", platform)
        } else {
            format!(
                "Platforms/{}.platform/Developer/SDKs/{}.sdk",
                platform, platform
            )
        };

        let suffix = "usr/include";
        let directory = format!("{}/{}/{}", prefix, infix, suffix);

        Ok(directory)
    }

    pub(crate) fn generate(manifest_dir: &str, out_dir: &str) {
        // on mac/iOS this will be modified, so it is marked as mutable.
        // on all other targets, this `mut` will be unused and the complainer compiles.t s
        #[allow(unused_mut)]
        let mut builder = bindgen::Builder::default()
            .header("godot_headers/gdnative_api_struct.gen.h")
            .whitelist_type("godot.*")
            .whitelist_function("godot.*")
            .whitelist_var("godot.*")
            .whitelist_type("GDNATIVE.*")
            .derive_default(true)
            .ignore_functions()
            .ctypes_prefix("libc")
            .clang_arg(format!("-I{}/godot_headers", manifest_dir));

        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

        // Workaround: x86_64 architecture is unsupported by the iPhone SDK, but cargo-lipo will
        // try to build it anyway. This leads to a clang error, so we'll skip the SDK.
        if target_os == "macos" || (target_os == "ios" && target_arch != "x86_64") {
            match osx_include_path() {
                Ok(osx_include_path) => {
                    builder = builder.clang_arg("-I").clang_arg(osx_include_path);
                }
                _ => {}
            }
        }

        // Workaround for https://github.com/rust-lang/rust-bindgen/issues/1211: manually set
        // target triple to `arm64-apple-ios` in place of `aarch64-apple-ios`.
        if target_arch == "aarch64" && target_os == "ios" {
            builder = builder.clang_arg("--target=arm64-apple-ios");
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

    #[derive(Debug, serde::Deserialize)]
    struct ApiRoot {
        core: Api,
        extensions: Vec<Api>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Api {
        name: Option<String>,
        #[serde(rename = "type")]
        type_: String,
        version: Version,
        next: Option<Box<Api>>,
        #[serde(rename = "api")]
        functions: Vec<Function>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Version {
        major: u32,
        minor: u32,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Function {
        name: String,
        return_type: String,
        arguments: Vec<Argument>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Argument {
        type_: String,
        name: String,
    }

    impl ApiRoot {
        fn all_apis(&self) -> Vec<&Api> {
            let mut result = Vec::new();
            result.extend(self.core.nexts());
            for extension in &self.extensions {
                result.extend(extension.nexts());
            }
            result
        }
    }

    impl Api {
        fn nexts(&self) -> Vec<&Api> {
            let mut result = Vec::new();
            let mut api = self;
            loop {
                result.push(api);
                if api.next.is_none() {
                    break;
                }
                api = &api.next.as_ref().unwrap();
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

    fn godot_api_struct_ident(type_: &str, version_major: u32, version_minor: u32) -> Ident {
        match (type_, version_major, version_minor) {
            ("CORE", 1, 0) => format_ident!("godot_gdnative_core_api_struct"),
            ("CORE", maj, min) => format_ident!("godot_gdnative_core_{}_{}_api_struct", maj, min),
            ("NATIVESCRIPT", 1, 0) => format_ident!("godot_gdnative_ext_nativescript_api_struct"),
            ("NATIVESCRIPT", maj, min) => {
                format_ident!("godot_gdnative_ext_nativescript_{}_{}_api_struct", maj, min)
            }
            ("PLUGINSCRIPT", 1, 0) => format_ident!("godot_gdnative_ext_pluginscript_api_struct"),
            ("ANDROID", 1, 0) => format_ident!("godot_gdnative_ext_android_api_struct"),
            ("ARVR", 1, 1) => format_ident!("godot_gdnative_ext_arvr_api_struct"),
            ("VIDEODECODER", 0, 1) => format_ident!("godot_gdnative_ext_videodecoder_api_struct"),
            ("NET", 3, 1) => format_ident!("godot_gdnative_ext_net_api_struct"),
            api => panic!("Unknown API type and version: {:?}", api),
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
            other => panic!("Unknown API type: {:?}", other),
        }
    }

    pub fn generate(
        from_json: &dyn AsRef<path::Path>,
        to: &dyn AsRef<path::Path>,
        file_name: &str,
    ) {
        let from_json = from_json.as_ref();
        let to = to.as_ref();
        let api_json_file = File::open(from_json).expect(&format!("No such file: {:?}", from_json));
        let api_root: ApiRoot = serde_json::from_reader(api_json_file)
            .expect(&"File ({:?}) does not contain expected JSON");
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
        let mut wrapper_file = File::create(to.join(file_name)).expect(&format!(
            "Couldn't create output file: {:?}",
            to.join(file_name)
        ));
        write!(wrapper_file, "{}", wrapper).unwrap();
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
        let mut constructed_struct_fields = TokenStream::new();
        for api in api.all_apis() {
            let i = api.macro_ident();
            let gd_api_type = api.godot_api_type();
            let v_maj = api.version.major;
            let v_min = api.version.minor;
            let gd_api_struct = api.godot_api_struct();
            godot_apis.extend(quote! {
                let #i = find_api_ptr(core_api_struct, #gd_api_type, #v_maj, #v_min) as *const #gd_api_struct;
            });
            for function in &api.functions {
                let function_name = function.rust_name();
                let expect_msg = format!(
                    "API function missing: {}.{}",
                    api.godot_api_struct(),
                    function_name
                );
                constructed_struct_fields.extend(quote! {
                    #function_name: (*#i).#function_name.expect(#expect_msg),
                });
            }
        }
        quote! {
            pub unsafe fn from_raw(core_api_struct: *const godot_gdnative_core_api_struct) -> Self {
                #godot_apis
                GodotApi{
                    #constructed_struct_fields
                }
            }
        }
    }

    fn parse_c_type(mut c_type: &str) -> (bool, i8, &str) {
        let is_const = c_type.starts_with("const ");
        if is_const {
            c_type = c_type.trim_start_matches("const ");
        }
        let mut ptr_count = 0;
        if c_type.ends_with("**") {
            ptr_count = 2;
            c_type = c_type.trim_end_matches("**");
        } else if c_type.ends_with("*") {
            ptr_count = 1;
            c_type = c_type.trim_end_matches("*");
        }
        (is_const, ptr_count, c_type.trim())
    }

    fn c_type_to_rust_type(c_type: &str) -> TokenStream {
        let (is_const, ptr_count, base_c_type) = parse_c_type(c_type);
        let rust_ptrs = match (ptr_count, is_const) {
            (0, _) => quote!(),
            (1, true) => quote!(*const ),
            (1, false) => quote!(*mut ),
            (2, true) => quote!(*mut *const ),
            _ => panic!("Unknown C type (Too many pointers?): {:?}", c_type),
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

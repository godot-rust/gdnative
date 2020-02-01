use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    header_binding::generate(&manifest_dir, &out_dir);

    api_wrapper::generate(&manifest_dir, &out_dir);
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
    #[derive(Debug, serde::Deserialize)]
    struct ApiDescription<'a> {
        #[serde(borrow)]
        core: CoreApi<'a>,
        #[serde(borrow)]
        extensions: Vec<ExtensionApi<'a>>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct CoreApi<'a> {
        #[serde(rename = "type")]
        type_: &'a str,
        version: Version,
        #[serde(borrow)]
        next: Option<Box<CoreApi<'a>>>,
        #[serde(borrow)]
        api: Vec<ApiFunction<'a>>,
    }

    impl<'a> CoreApi<'a> {
        fn struct_type_name(&self, root: bool) -> String {
            if root {
                "godot_gdnative_core_api_struct".to_string()
            } else {
                format!(
                    "godot_gdnative_core_{}_{}_api_struct",
                    self.version.major, self.version.minor
                )
            }
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct ExtensionApi<'a> {
        name: Option<&'a str>,
        #[serde(rename = "type")]
        type_: &'a str,
        version: Version,
        #[serde(borrow)]
        next: Option<Box<ExtensionApi<'a>>>,
        #[serde(borrow)]
        api: Vec<ApiFunction<'a>>,
    }

    impl<'a> ExtensionApi<'a> {
        fn struct_type_name(&self, root: bool) -> String {
            if root {
                format!(
                    "godot_gdnative_ext_{}_api_struct",
                    self.type_.to_lowercase()
                )
            } else {
                format!(
                    "godot_gdnative_ext_{}_{}_{}_api_struct",
                    self.type_.to_lowercase(),
                    self.version.major,
                    self.version.minor
                )
            }
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct Version {
        major: i64,
        minor: i64,
    }

    type ArgumentType<'a> = &'a str;
    type ArgumentName<'a> = &'a str;

    #[derive(Debug, serde::Deserialize)]
    struct ApiFunction<'a> {
        name: &'a str,
        return_type: &'a str,
        #[serde(borrow)]
        arguments: Vec<(ArgumentType<'a>, ArgumentName<'a>)>,
    }

    pub(crate) fn generate(manifest_dir: &str, out_dir: &str) {
        let api_json_path = format!("{}/godot_headers/gdnative_api.json", manifest_dir);

        let contents =
            std::fs::read_to_string(api_json_path).expect("Unable to read gdnative_api.json");

        let desc: ApiDescription<'_> =
            serde_json::from_str(&contents).expect("Unable to parse gdnative_api.json");

        let mut file = std::fs::File::create(format!("{}/api_wrapper.rs", out_dir)).unwrap();
        assemble_wrapper(&mut file, desc).unwrap();
    }

    fn assemble_wrapper<W: std::io::Write>(
        output: &mut W,
        desc: ApiDescription<'_>,
    ) -> std::io::Result<()> {
        writeln!(
            output,
            r#"
def_api! {{
struct GodotApi {{
    core {{"#
        )?;

        assemble_core_def(output, &desc.core, true)?;

        writeln!(
            output,
            r#"
    }}
    extensions {{"#,
        )?;

        assemble_ext_def(output, &desc)?;

        writeln!(
            output,
            r#"
    }}
}}
}}"#
        )?;
        Ok(())
    }

    fn assemble_core_def<W: std::io::Write>(
        output: &mut W,
        api: &CoreApi<'_>,
        root: bool,
    ) -> std::io::Result<()> {
        let label = format!("core_{}_{}", api.version.major, api.version.minor);

        writeln!(
            output,
            "        {label}({struct_type}, {vmajor}, {vminor}) {{",
            label = label,
            struct_type = api.struct_type_name(root),
            vmajor = api.version.major,
            vminor = api.version.minor,
        )?;

        for func in &api.api {
            assemble_api_func_type(output, func)?;
        }

        writeln!(output, "        }}")?;

        if let Some(api) = &api.next {
            assemble_core_def(output, &*api, false)?;
        }

        Ok(())
    }

    fn assemble_ext_def<W: std::io::Write>(
        output: &mut W,
        api: &ApiDescription<'_>,
    ) -> std::io::Result<()> {
        for ext in &api.extensions {
            // TODO feature selection
            //
            // skip android for now, it uses types from headers we are not
            // binding to.
            if ext.type_ == "ANDROID" {
                continue;
            }
            assemble_extension_ext_def(output, ext, true)?;
        }
        Ok(())
    }

    fn assemble_extension_ext_def<W: std::io::Write>(
        output: &mut W,
        api: &ExtensionApi<'_>,
        root: bool,
    ) -> std::io::Result<()> {
        let label = format!(
            "{}_{}_{}",
            api.type_.to_lowercase(),
            api.version.major,
            api.version.minor
        );

        let key = format!("GDNATIVE_API_TYPES_GDNATIVE_EXT_{}", api.type_);

        writeln!(
            output,
            "        {label}({key}, {struct_type}, {vmaj}, {vmin}) {{",
            label = label,
            key = key,
            struct_type = api.struct_type_name(root),
            vmaj = api.version.major,
            vmin = api.version.minor,
        )?;

        for func in &api.api {
            assemble_api_func_type(output, func)?;
        }

        writeln!(output, "        }}")?;

        if let Some(api) = &api.next {
            assemble_extension_ext_def(output, &*api, false)?;
        }
        Ok(())
    }

    fn assemble_api_func_type<W: std::io::Write>(
        output: &mut W,
        func: &ApiFunction<'_>,
    ) -> std::io::Result<()> {
        let name = func.name;

        writeln!(
            output,
            r#"            pub {name}: unsafe extern "C" fn("#,
            name = name,
        )?;

        // write arguments
        for (arg_type, arg_name) in &func.arguments {
            write!(output, r#"                {}: "#, arg_name)?;
            write_converted_c_type(output, arg_type)?;
            writeln!(output, ",")?;
        }

        write!(output, r#"            ) -> "#)?;
        write_converted_c_type(output, func.return_type)?;
        writeln!(output, r#","#)?;

        Ok(())
    }

    fn write_converted_c_type<W: std::io::Write>(output: &mut W, ty: &str) -> std::io::Result<()> {
        let mut text = ty;

        let is_const = text.starts_with("const ");

        if is_const {
            text = ty.trim_start_matches("const ");
        }

        let is_double_pointer = text.ends_with("**");
        let is_single_pointer = text.ends_with("*");

        if is_double_pointer {
            text = text.trim_end_matches(" **");
            text = text.trim_end_matches("**");
        } else if is_single_pointer {
            text = text.trim_end_matches(" *");
            text = text.trim_end_matches("*");
        } else {
        }

        let text = match text {
            "void" => {
                // a pointer to void is a different type than a void return
                if is_double_pointer || is_single_pointer {
                    "std::os::raw::c_void"
                } else {
                    "()"
                }
            }
            "char" => "std::os::raw::c_char",
            "signed char" => "std::os::raw::c_schar",
            "double" => "std::os::raw::c_double",
            "float" => "std::os::raw::c_float",
            "size_t" => "usize",
            "int" => "std::os::raw::c_int",
            "int8_t" => "i8",
            "uint8_t" => "u8",
            "int32_t" => "i32",
            "uint32_t" => "u32",
            "int64_t" => "i64",
            "uint64_t" => "u64",
            _ => text,
        };

        // there should be no spaces after the const and pointer modifiers are
        // dropped.
        debug_assert!(!text.contains(' '));

        let ptr_prefix = if is_double_pointer {
            if is_const {
                "*mut *const "
            } else {
                "*mut *mut "
            }
        } else if is_single_pointer {
            if is_const {
                "*const "
            } else {
                "*mut "
            }
        } else {
            ""
        };

        write!(output, "{}{}", ptr_prefix, text)
    }
}

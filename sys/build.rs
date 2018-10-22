
extern crate bindgen;

use std::env;
use std::path::PathBuf;

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn osx_include_path() -> Result<String, std::io::Error> {
    use std::process::Command;

    let output = Command::new("xcode-select").arg("-p").output()?.stdout;
    let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcode-select`");
    let prefix = prefix_str.trim_right();

    let platform = if cfg!(target_os = "macos") {
        "MacOSX"
    } else if cfg!(target_os = "ios") {
        "iPhoneOS"
    } else {
        unreachable!();
    };

    let infix = if prefix == "/Library/Developer/CommandLineTools" {
        format!("SDKs/{}.sdk", platform)
    } else {
        format!("Platforms/{}.platform/Developer/SDKs/{}.sdk", platform, platform)
    };

    let suffix = "usr/include";
    let directory = format!("{}/{}/{}", prefix, infix, suffix);

    Ok(directory)
}

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bindings = {
        let mut builder = bindgen::Builder::default()
            .header("godot_headers/gdnative_api_struct.gen.h")
            .whitelisted_type("godot.*")
            .whitelisted_function("godot.*")
            .whitelisted_var("godot.*")
            .whitelisted_type("GDNATIVE.*")
            .derive_default(true)
            .ignore_functions()
            .ctypes_prefix("libc")
            .clang_arg(format!("-I{}/godot_headers", dir));

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        match osx_include_path() {
            Ok(osx_include_path) => {
                builder = builder.clang_arg("-I").clang_arg(osx_include_path);
            },
            _ => {},
        }

        builder.generate()
            .expect("Unable to generate bindings")
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
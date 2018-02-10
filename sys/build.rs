
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let bindings = bindgen::Builder::default()
        .header("godot_headers/gdnative_api_struct.gen.h")
        .whitelisted_type("godot.*")
        .whitelisted_function("godot.*")
        .whitelisted_var("godot.*")
        .whitelisted_type("GDNATIVE.*")
        .derive_default(true)
        .ignore_functions()
        .ctypes_prefix("libc")
        .clang_arg(format!("-I{}/godot_headers", dir))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
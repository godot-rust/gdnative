extern crate bindgen;

use std::env;
use std::path::Path;

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  bindgen::builder()
    .header("godot_headers/godot.h")
    .no_unstable_rust()
    .generate().unwrap()
    .write_to_file(Path::new("src/godot_core_api.rs")).unwrap();
}

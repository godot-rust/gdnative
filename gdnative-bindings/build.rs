use gdnative_bindings_generator::*;

use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut types_output =
        BufWriter::new(File::create(out_path.join("bindings_types.rs")).unwrap());
    let mut traits_output =
        BufWriter::new(File::create(out_path.join("bindings_traits.rs")).unwrap());
    let mut methods_output =
        BufWriter::new(File::create(out_path.join("bindings_methods.rs")).unwrap());

    // gdnative-core already implements all dependencies of Object
    let to_ignore = { strongly_connected_components(&Api::new(), "Object", None) };

    generate_bindings(
        &mut types_output,
        &mut traits_output,
        &mut methods_output,
        Some(to_ignore),
    )
    .unwrap();
}

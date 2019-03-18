use gdnative_bindings_generator::*;

use std::path::PathBuf;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut output = File::create(out_path.join("bindings.rs")).unwrap();

    // gdnative-core already implements all dependencies of Object
    let to_ignore = {
        strongly_connected_components(
            &Api::new(),
            "Object",
            None,
        )
    };

    generate_bindings(&mut output, Some(to_ignore)).unwrap();
}
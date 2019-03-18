use gdnative_bindings_generator::*;
use std::path::PathBuf;
use std::env;
use std::fs::File;

fn main() {

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut output = File::create(out_path.join("core_types.rs")).unwrap();

    let classes = strongly_connected_components(
        &Api::new(),
        "Object",
        None,
    );

    generate_imports(&mut output).unwrap();

    for class in classes {
        generate_class(&mut output, &class).unwrap();
    }
}

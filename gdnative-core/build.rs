use gdnative_bindings_generator::*;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let core_types_rs = out_path.join("core_types.rs");
    let core_traits_rs = out_path.join("core_traits.rs");
    let core_methods_rs = out_path.join("core_methods.rs");

    let mut types_output = File::create(&core_types_rs).unwrap();
    let mut traits_output = File::create(&core_traits_rs).unwrap();
    let mut methods_output = File::create(&core_methods_rs).unwrap();

    let classes = strongly_connected_components(&Api::new(), "Object", None);

    for class in classes {
        generate_class(
            &mut types_output,
            &mut traits_output,
            &mut methods_output,
            &class,
        )
        .unwrap();
    }

    // Close the files
    drop(types_output);
    drop(traits_output);
    drop(methods_output);

    for file in &[core_types_rs, core_traits_rs, core_methods_rs] {
        let output = Command::new("rustup")
            .arg("run")
            .arg("stable")
            .arg("rustfmt")
            .arg("--edition")
            .arg("2018")
            .arg(file)
            .output()
            .unwrap();
        eprintln!("Formatting output: {:?}", output);
    }
}

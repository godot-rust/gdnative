use gdnative_bindings_generator::*;

use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings_types_rs = out_path.join("bindings_types.rs");
    let bindings_traits_rs = out_path.join("bindings_traits.rs");
    let bindings_methods_rs = out_path.join("bindings_methods.rs");

    let mut types_output = BufWriter::new(File::create(&bindings_types_rs).unwrap());
    let mut traits_output = BufWriter::new(File::create(&bindings_traits_rs).unwrap());
    let mut methods_output = BufWriter::new(File::create(&bindings_methods_rs).unwrap());

    // gdnative-core already implements all dependencies of Object
    let to_ignore = { strongly_connected_components(&Api::new(), "Object", None) };

    generate_bindings(
        &mut types_output,
        &mut traits_output,
        &mut methods_output,
        Some(to_ignore),
    )
    .unwrap();

    drop(types_output);
    drop(traits_output);
    drop(methods_output);

    for file in &[bindings_types_rs, bindings_traits_rs, bindings_methods_rs] {
        print!(
            "Formatting generated file: {}... ",
            file.file_name().map(|s| s.to_str()).flatten().unwrap()
        );
        match Command::new("rustup")
            .arg("run")
            .arg("stable")
            .arg("rustfmt")
            .arg("--edition=2018")
            .arg(file)
            .output()
        {
            Ok(_) => println!("Done"),
            Err(err) => {
                println!("Failed");
                println!("Error: {}", err);
            }
        }
    }
}

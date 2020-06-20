use gdnative_bindings_generator::*;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let output_rs = out_path.join("generated.rs");

    {
        let mut output = BufWriter::new(File::create(&output_rs).unwrap());

        // gdnative-core already implements all dependencies of Object
        let to_ignore = strongly_connected_components(&Api::new(), "Object", None);

        let code = generate_bindings(Some(to_ignore));
        write!(&mut output, "{}", code).unwrap();
    }

    print!(
        "Formatting generated file: {}... ",
        output_rs.file_name().map(|s| s.to_str()).flatten().unwrap()
    );
    match Command::new("rustup")
        .arg("run")
        .arg("stable")
        .arg("rustfmt")
        .arg("--edition=2018")
        .arg(output_rs)
        .output()
    {
        Ok(_) => println!("Done"),
        Err(err) => {
            println!("Failed");
            println!("Error: {}", err);
        }
    }
}

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

        let api = Api::new();

        let code = generate_bindings(&api);
        write!(&mut output, "{}", code).unwrap();
    }

    if cfg!(feature = "formatted") {
        format_file(&output_rs);
    }

    // build.rs will automatically be recompiled and run if it's dependencies are updated.
    // Ignoring all but build.rs will keep from needless rebuilds.
    // Manually rebuilding the crate will ignore this.
    println!("cargo:rerun-if-changed=build.rs");
}

fn format_file(output_rs: &PathBuf) {
    print!(
        "Formatting generated file: {}... ",
        output_rs.file_name().and_then(|s| s.to_str()).unwrap()
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

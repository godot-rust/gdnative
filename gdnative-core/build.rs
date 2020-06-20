use gdnative_bindings_generator::*;

use quote::quote;

use std::env;
use std::fs::File;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let generated_rs = out_path.join("generated.rs");

    {
        let mut output = File::create(&generated_rs).unwrap();

        let classes = strongly_connected_components(&Api::new(), "Object", None);

        let code = classes.iter().map(|class| generate_class(&class));
        let code = quote! {
            #(#code)*
        };
        write!(&mut output, "{}", code).unwrap();
    }

    for file in &[generated_rs] {
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

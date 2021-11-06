use gdnative_bindings_generator::*;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let api_data = std::fs::read_to_string("api.json").expect("Unable to read api.json");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let generated_rs = out_path.join("generated.rs");
    let icalls_rs = out_path.join("icalls.rs");

    let api = Api::new(&api_data);
    let docs = GodotXmlDocs::new("docs");
    let binding_res = generate_bindings(&api, Some(&docs));

    {
        let mut output = BufWriter::new(File::create(&generated_rs).unwrap());

        generate(&out_path, &mut output, &binding_res);
    }

    {
        let mut output = BufWriter::new(File::create(&icalls_rs).unwrap());

        write!(&mut output, "{}", binding_res.icalls).unwrap();
    }

    if cfg!(feature = "formatted") {
        format_file(&generated_rs);
        format_file(&icalls_rs);
    }

    // build.rs will automatically be recompiled and run if it's dependencies are updated.
    // Ignoring all but build.rs will keep from needless rebuilds.
    // Manually rebuilding the crate will ignore this.
    println!("cargo:rerun-if-changed=docs/");
    println!("cargo:rerun-if-changed=api.json");
    println!("cargo:rerun-if-changed=build.rs");
}

/// Output all the class bindings into the `generated.rs` file.
#[cfg(not(feature = "one_class_one_file"))]
fn generate(
    _out_path: &std::path::Path,
    generated_file: &mut BufWriter<File>,
    binding_res: &BindingResult,
) {
    for (class_name, code) in &binding_res.class_bindings {
        write!(
            generated_file,
            r#"
            pub mod {mod_name} {{
                use super::*;
                {content}
            }}
            pub use crate::generated::{mod_name}::private::{class_name};
            "#,
            mod_name = module_name_from_class_name(class_name),
            class_name = class_name,
            content = code,
        )
        .unwrap();
    }
}

/// Output one file for each class and add `mod` and `use` declarations in
/// the `generated.rs` file.
#[cfg(feature = "one_class_one_file")]
fn generate(
    out_path: &std::path::Path,
    generated_file: &mut BufWriter<File>,
    binding_res: &BindingResult,
) {
    for (class_name, code) in &binding_res.class_bindings {
        let mod_name = module_name_from_class_name(class_name);

        let mod_path = out_path.join(format!("{}.rs", mod_name));
        let mut mod_output = BufWriter::new(File::create(&mod_path).unwrap());

        write!(
            &mut mod_output,
            r#"use super::*;
            {content}"#,
            content = code,
        )
        .unwrap();

        drop(mod_output);

        if cfg!(feature = "formatted") {
            format_file(&mod_path);
        }

        writeln!(
            generated_file,
            r#"
            #[path = {:?}]
            pub mod {mod_name};
            pub use crate::generated::{mod_name}::private::{class_name};
            "#,
            mod_path.display(),
            mod_name = mod_name,
            class_name = class_name,
        )
        .unwrap();
    }
}

fn format_file(output_rs: &Path) {
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

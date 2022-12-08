use gdnative_bindings_generator as gen;

use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};

fn main() {
    let just_generated_api = gen::generate_json_if_needed();

    let api_data = std::fs::read_to_string("api.json").expect("Unable to read api.json");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let generated_rs = out_path.join("generated.rs");
    let icalls_rs = out_path.join("icalls.rs");

    let api = gen::Api::new(&api_data);
    let docs = gen::GodotXmlDocs::new("docs");
    let binding_res = gen::generate_bindings(&api, Some(&docs));

    {
        let mut output = BufWriter::new(File::create(&generated_rs).unwrap());

        generate(&out_path, &mut output, &binding_res);
    }

    {
        let mut output = BufWriter::new(File::create(&icalls_rs).unwrap());

        write!(output, "{}", binding_res.icalls).unwrap();
    }

    format_file_if_needed(&generated_rs);
    format_file_if_needed(&icalls_rs);

    // build.rs will automatically be recompiled and run if its dependencies are updated.
    // Ignoring everything but build.rs will avoid needless rebuilds.
    // Manually rebuilding the crate does not affect this.
    println!("cargo:rerun-if-changed=build.rs");

    // Avoid endless recompiling, if this script generates api.json and docs
    if !just_generated_api {
        println!("cargo:rerun-if-changed=docs/");
        println!("cargo:rerun-if-changed=api.json");
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Feature 'one-class-one-file'

/// Output all the class bindings into the `generated.rs` file.
#[cfg(not(feature = "one-class-one-file"))]
fn generate(
    _out_path: &std::path::Path,
    generated_file: &mut BufWriter<File>,
    binding_res: &gen::BindingResult,
) {
    // Note: 'use super::*;' needs to be after content, as the latter may contain #![doc] attributes,
    // which need to be at the beginning of the module
    for (class, code) in &binding_res.class_bindings {
        let modifier = if class.has_related_module() {
            "pub"
        } else {
            "pub(crate)"
        };
        write!(
            generated_file,
            r#"
            {modifier} mod {mod_name} {{
                {content}
                use super::*;
            }}
            pub use crate::generated::{mod_name}::private::{class_name};
            "#,
            modifier = modifier,
            mod_name = gen::module_name_from_class_name(&class.name),
            class_name = class.name,
            content = code,
        )
        .unwrap();
    }
}

/// Output one file for each class and add `mod` and `use` declarations in
/// the `generated.rs` file.
#[cfg(feature = "one-class-one-file")]
fn generate(
    out_path: &std::path::Path,
    generated_file: &mut BufWriter<File>,
    binding_res: &gen::BindingResult,
) {
    for (class, code) in &binding_res.class_bindings {
        let mod_name = gen::module_name_from_class_name(&class.name);

        let mod_path = out_path.join(format!("{mod_name}.rs"));
        let mut mod_output = BufWriter::new(File::create(&mod_path).unwrap());

        write!(
            &mut mod_output,
            r#"
            {code}
            use super::*;
            "#,
        )
        .unwrap();

        drop(mod_output);

        format_file_if_needed(&mod_path);

        let modifier = if class.has_related_module() {
            "pub"
        } else {
            "pub(crate)"
        };
        writeln!(
            generated_file,
            r#"
            #[path = {:?}]
            {modifier} mod {mod_name};
            pub use crate::generated::{mod_name}::private::{class_name};
            "#,
            mod_path.display(),
            modifier = modifier,
            mod_name = mod_name,
            class_name = class.name,
        )
        .unwrap();
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Feature 'formatted'

#[cfg(feature = "formatted")]
fn format_file_if_needed(output_rs: &Path) {
    print!(
        "Formatting generated file: {}... ",
        output_rs.file_name().and_then(|s| s.to_str()).unwrap()
    );

    let output = std::process::Command::new("rustup")
        .arg("run")
        .arg("stable")
        .arg("rustfmt")
        .arg("--edition=2021")
        .arg(output_rs)
        .output();

    match output {
        Ok(_) => println!("Done."),
        Err(err) => {
            println!("Failed.");
            println!("Error: {err}");
        }
    }
}

#[cfg(not(feature = "formatted"))]
fn format_file_if_needed(_output_rs: &Path) {}

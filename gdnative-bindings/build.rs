use gdnative_bindings_generator::*;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let just_generated_api = generate_api_if_needed();

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
    println!("cargo:rerun-if-changed=build.rs");

    // Avoid endless recompiling, if this script generates api.json
    if !just_generated_api {
        println!("cargo:rerun-if-changed=api.json");
    }
}

/// Output all the class bindings into the `generated.rs` file.
#[cfg(not(feature = "one_class_one_file"))]
fn generate(
    _out_path: &std::path::Path,
    generated_file: &mut BufWriter<File>,
    binding_res: &BindingResult,
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
            mod_name = module_name_from_class_name(&class.name),
            class_name = class.name,
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
    for (class, code) in &binding_res.class_bindings {
        let mod_name = module_name_from_class_name(&class.name);

        let mod_path = out_path.join(format!("{}.rs", mod_name));
        let mut mod_output = BufWriter::new(File::create(&mod_path).unwrap());

        write!(
            &mut mod_output,
            r#"
            {content}
            use super::*;
            "#,
            content = code,
        )
        .unwrap();

        drop(mod_output);

        if cfg!(feature = "formatted") {
            format_file(&mod_path);
        }

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

#[cfg(feature = "custom-godot")]
fn generate_api_if_needed() -> bool {
    let source: String;
    let godot_bin: PathBuf;

    if let Ok(string) = env::var("GODOT_BIN") {
        source = format!("GODOT_BIN executable '{}'", string);
        godot_bin = PathBuf::from(string);
    } else if let Ok(path) = which::which("godot") {
        source = "executable 'godot'".to_string();
        godot_bin = path;
    } else {
        panic!(
            "Feature 'custom-godot' requires an accessible 'godot' executable or \
             a GODOT_BIN environment variable (with the path to the executable)."
        );
    };

    // TODO call 'godot --version' and ensure >= 3.2 && < 4.0

    let status = Command::new(godot_bin)
        .arg("--gdnative-generate-json-api")
        .arg("api.json")
        .status()
        .unwrap_or_else(|err| panic!("Failed to invoke {}; error {}", source, err));

    assert!(
        status.success(),
        "Custom Godot command exited with status {}",
        status.code().map_or("?".to_string(), |f| f.to_string())
    );

    true
}

#[cfg(not(feature = "custom-godot"))]
fn generate_api_if_needed() -> bool {
    false
}

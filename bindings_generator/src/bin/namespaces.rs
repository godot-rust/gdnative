extern crate gdnative_bindings_generator;
extern crate serde;
extern crate serde_json;

use gdnative_bindings_generator::*;
use gdnative_bindings_generator::json::*;
use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
    let in_path = env::args().nth(1).unwrap();
    let out_path = env::args().nth(2).unwrap();

    let mut output = File::create(&out_path).unwrap();

    generate_crate_namespaces(
        File::open(&in_path).unwrap(),
        &mut output,
    ).unwrap();
}

pub fn generate_crate_namespaces(
    api_description: File,
    output: &mut File,
) -> GeneratorResult {
    let classes: Vec<GodotClass> = serde_json::from_reader(api_description).expect("Failed to parse the API description");

    writeln!(output, "{{")?;
    for class in &classes {
        let crate_type = get_crate(&classes, class);
        writeln!(output, r#"    "{}": "{:?}","#,
            class.name,
            crate_type,
        )?;
    }
    writeln!(output, r#"    "_bindings_sentinel_": "Unknown""#)?;
    writeln!(output, "}}")?;

    Ok(())
}

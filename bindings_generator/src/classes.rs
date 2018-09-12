use json::*;
use std::fs::File;
use std::io::Write;
use GeneratorResult;
use heck::CamelCase;

pub fn generate_class_struct(output: &mut File, class: &GodotClass) -> GeneratorResult {
    if !class.is_refcounted() {
        writeln!(output, "#[derive(Copy, Clone)]")?;
    }

    writeln!(output,
r#"#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct {name} {{
    #[doc(hidden)]
    pub this: *mut sys::godot_object,
}}
"#,
        name = class.name
    )?;

    Ok(())
}

pub fn generate_enum(output: &mut File, class: &GodotClass, e: &Enum) -> GeneratorResult {
    // TODO: check whether the start of the variant name is
    // equal to the end of the enum name and if so don't repeat it
    // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.

    let mut values: Vec<(&String, &u32)> = e.values.iter().collect();
    values.sort_by(|a, b|{ a.1.cmp(&b.1) });

    writeln!(output,
r#"#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum {class_name}{enum_name} {{"#,
        class_name = class.name, enum_name = e.name
    )?;

    for &(key, val) in &values {
        let key = key.as_str().to_camel_case();
        writeln!(output, r#"    {key} = {val},"#, key = key, val = val)?;
    }
    writeln!(output, "}}")?;

    Ok(())
}

use json::*;
use std::fs::File;
use std::io::Write;

use heck::CamelCase;

pub fn generate_class_struct(output: &mut File, class: &GodotClass) {
    if !class.is_refcounted() {
        writeln!(output, "#[derive(Copy, Clone)]").unwrap();
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
    ).unwrap();
}

pub fn generate_enum(output: &mut File, class: &GodotClass, e: &Enum) {
    // TODO: check whether the start of the variant name is
    // equal to the end of the enum name and if so don't repeat it
    // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.
    writeln!(output, r#"
#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum {class_name}{enum_name} {{
"#,
        class_name = class.name, enum_name = e.name
    ).unwrap();

    for (key, val) in &e.values {
        let key = key.as_str().to_camel_case();
        writeln!(output, r#"    {key} = {val},"#, key = key, val = val).unwrap();
    }
    writeln!(output, r#"
}}"#
    ).unwrap();
}

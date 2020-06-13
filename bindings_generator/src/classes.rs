use crate::api::*;
use crate::GeneratorResult;
use heck::CamelCase;
use std::io::Write;

pub fn generate_class_struct(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    // FIXME(#390): non-RefCounted types should not be Clone
    if !class.is_refcounted() {
        writeln!(output, "#[derive(Copy, Clone)]")?;
    }

    writeln!(
        output,
        r#"#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct {name} {{
    this: *mut sys::godot_object,
}}
"#,
        name = class.name
    )?;

    Ok(())
}

pub fn generate_class_constants(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    if class.constants.is_empty() {
        return Ok(());
    }

    writeln!(output, "/// Constants")?;
    writeln!(output, "#[allow(non_upper_case_globals)]")?;
    writeln!(output, "impl {} {{", class.name)?;

    for (name, value) in &class.constants {
        writeln!(
            output,
            "    pub const {name}: i64 = {value};",
            name = name,
            value = value,
        )?;
    }

    writeln!(output, "}}")?;
    Ok(())
}

#[derive(Copy, Clone, PartialEq)]
struct EnumReference<'a> {
    class: &'a str,
    enum_name: &'a str,
    enum_variant: &'a str,
}

const ENUM_VARIANTS_TO_SKIP: &[EnumReference<'static>] = &[
    EnumReference {
        class: "MultiplayerAPI",
        enum_name: "RPCMode",
        enum_variant: "RPC_MODE_SLAVE",
    },
    EnumReference {
        class: "MultiplayerAPI",
        enum_name: "RPCMode",
        enum_variant: "RPC_MODE_SYNC",
    },
    EnumReference {
        class: "TextureLayered",
        enum_name: "Flags",
        enum_variant: "FLAGS_DEFAULT",
    },
    EnumReference {
        class: "CameraServer",
        enum_name: "FeedImage",
        enum_variant: "FEED_YCBCR_IMAGE",
    },
    EnumReference {
        class: "CameraServer",
        enum_name: "FeedImage",
        enum_variant: "FEED_Y_IMAGE",
    },
];

pub fn generate_enum(output: &mut impl Write, class: &GodotClass, e: &Enum) -> GeneratorResult {
    // TODO: check whether the start of the variant name is
    // equal to the end of the enum name and if so don't repeat it
    // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.

    let mut values: Vec<(&String, &i64)> = e.values.iter().collect();
    values.sort_by(|a, b| a.1.cmp(&b.1));

    writeln!(
        output,
        r#"#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum {class_name}{enum_name} {{"#,
        class_name = class.name,
        enum_name = e.name
    )?;

    let mut previous_value = None;

    for &(key, val) in &values {
        let val = *val as u64 as u32;

        // Use lowercase to test because of different CamelCase conventions (Msaa/MSAA, etc.).
        let enum_ref = EnumReference {
            class: class.name.as_str(),
            enum_name: e.name.as_str(),
            enum_variant: key.as_str(),
        };

        if ENUM_VARIANTS_TO_SKIP.contains(&enum_ref) {
            continue;
        }

        // Check if the value is a duplicate. This is fine because `values` is already sorted by value.
        if Some(val) == previous_value.replace(val) {
            println!(
                "cargo:warning=Enum variant {class}.{name}.{variant} skipped: duplicate value {value}",
                class = enum_ref.class,
                name = enum_ref.enum_name,
                variant = enum_ref.enum_variant,
                value = val,
            );
            continue;
        }

        let enum_name_without_mode = if e.name.ends_with("Mode") {
            e.name[0..(e.name.len() - 4)].to_lowercase()
        } else {
            e.name[..].to_lowercase()
        };
        let mut key = key.as_str().to_camel_case();
        if let Some(new_key) = try_remove_prefix(&key, &e.name) {
            key = new_key;
        } else if let Some(new_key) = try_remove_prefix(&key, &enum_name_without_mode) {
            key = new_key;
        }
        writeln!(output, r#"    {key} = {val},"#, key = key, val = val)?;
    }
    writeln!(output, "}}")?;

    Ok(())
}

fn try_remove_prefix(key: &str, prefix: &str) -> Option<String> {
    let key_lower = key.to_lowercase();
    if key_lower.starts_with(prefix)
        && !key
            .chars()
            .nth(prefix.len())
            .map_or(true, |c| c.is_numeric())
    {
        return Some(key[prefix.len()..].to_string());
    }

    None
}

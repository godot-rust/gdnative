use crate::api::*;
use crate::methods;
use crate::special_methods;
use crate::GeneratorResult;

use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::collections::HashSet;
use std::io::Write;

pub fn generate_class_struct(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    // FIXME(#390): non-RefCounted types should not be Clone
    let derive_copy = if !class.is_refcounted() {
        quote! {
            #[derive(Copy, Clone)]
        }
    } else {
        TokenStream::new()
    };

    let class_name = format_ident!("{}", &class.name);

    let code = quote! {
        #derive_copy
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub struct #class_name {
            this: *mut sys::godot_object,
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_class_impl(
    output: &mut impl Write,
    api: &Api,
    class: &GodotClass,
) -> GeneratorResult {
    let class_singleton = if class.singleton {
        special_methods::generate_singleton_getter(class)
    } else {
        Default::default()
    };

    let class_singleton_getter = if class.name == "GDNativeLibrary" {
        special_methods::generate_gdnative_library_singleton_getter(class)
    } else {
        Default::default()
    };

    let class_instanciable = if class.instantiable {
        if class.is_refcounted() {
            special_methods::generate_reference_ctor(class)
        } else {
            special_methods::generate_non_reference_ctor(class)
        }
    } else {
        Default::default()
    };

    let mut method_set = HashSet::default();

    let class_methods = methods::generate_methods(
        &api,
        &mut method_set,
        &class.name,
        class.is_pointer_safe(),
        true,
    );

    let class_upcast =
        special_methods::generate_upcast(&api, &class.base_class, class.is_pointer_safe());

    let class_dynamic_cast = special_methods::generate_dynamic_cast(class);

    let class_name = format_ident!("{}", class.name);
    let struct_impl = quote! {
        impl #class_name {
            #class_singleton
            #class_singleton_getter
            #class_instanciable
            #class_methods
            #class_upcast
            #class_dynamic_cast
        }
    };

    generated_at!(output);
    write!(output, "{}", struct_impl)?;

    Ok(())
}

pub fn generate_class_constants(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    assert!(
        !class.constants.is_empty(),
        "Only call on class with constants."
    );

    let mut constants = TokenStream::new();

    let mut class_constants: Vec<(&ConstantName, &ConstantValue)> =
        class.constants.iter().collect();
    class_constants.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, value) in &class_constants {
        let name = format_ident!("{}", name);
        let constant = quote! {
            pub const #name: i64 = #value;
        };
        constants.extend(constant);
    }

    let class_name = format_ident!("{}", &class.name);

    let code = quote! {
        #[doc="Constants"]
        #[allow(non_upper_case_globals)]
        impl #class_name {
            #constants
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

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

pub fn generate_enums(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    // TODO: check whether the start of the variant name is
    // equal to the end of the enum name and if so don't repeat it
    // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.
    let mut enums: Vec<&Enum> = class.enums.iter().collect();
    enums.sort();
    let enums: Vec<TokenStream> = enums.iter().map(|e| {
        let mut enum_values = TokenStream::new();

        let mut values: Vec<(&String, &i64)> = e.values.iter().collect();
        values.sort_by(|a, b| {
            a.1.cmp(&b.1)
        });

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

            let key = format_ident!("{}", key);
            let output = quote! {
                #key = #val,
            };
            enum_values.extend(output);
        }

        let enum_name = format_ident!("{}{}", class.name, e.name);
        quote! {
            #[repr(u32)]
            #[allow(non_camel_case_types)]
            #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
            pub enum #enum_name {
                #enum_values
            }
        }
    }).collect();

    let code = quote! {
        #(#enums)*
    };

    generated_at!(output);
    write!(output, "{}", code)?;

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

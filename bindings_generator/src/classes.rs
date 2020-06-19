use crate::api::*;
use crate::methods;
use crate::special_methods;

use heck::{CamelCase as _, ShoutySnakeCase as _};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::collections::HashSet;

pub fn generate_class_struct(class: &GodotClass) -> TokenStream {
    // FIXME(#390): non-RefCounted types should not be Clone
    let derive_copy = if !class.is_refcounted() {
        quote! {
            #[derive(Copy, Clone)]
        }
    } else {
        TokenStream::new()
    };

    let class_name = format_ident!("{}", &class.name);

    quote! {
        #derive_copy
        #[allow(non_camel_case_types)]
        #[derive(Debug)]
        pub struct #class_name {
            this: *mut sys::godot_object,
        }
    }
}

pub fn generate_class_impl(api: &Api, class: &GodotClass) -> TokenStream {
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
    quote! {
        impl #class_name {
            #class_singleton
            #class_singleton_getter
            #class_instanciable
            #class_methods
            #class_upcast
            #class_dynamic_cast
        }
    }
}

pub fn generate_class_constants(class: &GodotClass) -> TokenStream {
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

    quote! {
        #[doc="Constants"]
        #[allow(non_upper_case_globals)]
        impl #class_name {
            #constants
        }
    }
}

fn generate_enum_variant_name<'a>(enum_name: &str, variant_name: &'a str) -> String {
    // Operating in SHOUTY_SNAKE_CASE to use more references and fewer allocations
    // Strip the Mode frome the end of the Enum name
    let enum_name = enum_name.to_shouty_snake_case().to_uppercase();

    let enum_name_without_mode = if enum_name.ends_with("_MODE") {
        &enum_name[0..(enum_name.len() - 5)]
    } else {
        &enum_name[..]
    };

    let enum_name_without_type = if enum_name.ends_with("_TYPE") {
        &enum_name[0..(enum_name.len() - 5)]
    } else {
        &enum_name[..]
    };

    if let Some(new_key) = try_remove_prefix(&variant_name, &enum_name) {
        new_key.to_uppercase()
    } else if let Some(new_key) = try_remove_prefix(&variant_name, &enum_name_without_mode) {
        new_key.to_uppercase()
    } else if let Some(new_key) = try_remove_prefix(&variant_name, &enum_name_without_type) {
        new_key.to_uppercase()
    } else {
        variant_name.to_uppercase()
    }
}

fn generate_enum_name(class_name: &str, enum_name: &str) -> String {
    // In order to not pollute the API with more Result types,
    // rename the Result enum used by Search to SearchResult
    // to_camel_case() is used to make the enums more Rust like.
    // DOFBlurQuality => DofBlurQuality
    match enum_name {
        "Result" => {
            let mut res = String::from(class_name);
            res.push_str(enum_name);
            res.to_camel_case()
        }
        _ => enum_name.to_camel_case(),
    }
}

pub fn generate_enums(class: &GodotClass) -> TokenStream {
    // TODO: check whether the start of the variant name is
    // equal to the end of the enum name and if so don't repeat it
    // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.
    let mut enums: Vec<&Enum> = class.enums.iter().collect();
    enums.sort();
    let enums = enums.iter().map(|e| {
        let enum_name = generate_enum_name(&class.name, &e.name);
        let typ_name = format_ident!("{}", enum_name);

        let mut values: Vec<(&String, &i64)> = e.values.iter().collect();
        values.sort_by(|a, b| a.1.cmp(&b.1));

        let consts = values.iter().filter_map(|(key, val)| {
            let key = generate_enum_variant_name(&e.name, &key);
            let variant = format_ident!("{}", key);
            Some(quote! {
                pub const #variant: #typ_name = #typ_name(#val);
            })
        });

        quote! {
            #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
            pub struct #typ_name(pub i64);

            impl #typ_name {
                #(#consts)*
            }
            impl Deref for #typ_name {
                type Target = i64;
                #[inline]
                fn deref(&self) -> &i64 {
                    &self.0
                }
            }

            impl From<i64> for #typ_name {
                #[inline]
                fn from(v: i64) -> Self {
                    Self(v)
                }
            }
            impl From<#typ_name> for i64 {
                #[inline]
                fn from(v: #typ_name) -> Self {
                    v.0
                }
            }
        }
    });

    quote! {
        #(#enums)*
    }
}

fn try_remove_prefix<'a>(variant: &'a str, class_name: &str) -> Option<&'a str> {
    // Check if the variant begins with the end of the class_name
    let mut variant_chunks = variant.split('_');
    let variant_beginning = variant_chunks.next()?;
    let variant_second = variant_chunks.next()?;

    if !variant_second
        .chars()
        .next()
        .map_or(true, |c| c.is_numeric())
    {
        let mut class_chunks = class_name.split('_');
        let class_end = class_chunks.next_back()?;
        if variant_beginning == class_end {
            let offset = class_end.len() + 1;
            return variant.get(offset..);
        }
        // english and plurals! SaverFlags::FLAG_RELATIVE_PATHS
        if class_end.ends_with('S') && variant_beginning == &class_end[..class_end.len() - 1] {
            // For easy sanity checking: cargo build -vv 2>&1 | rg "FOUND PLURAL"
            // eprintln!("FOUND PLURAL: {} and {}", &class_name, &variant);
            let offset = class_end.len();
            return variant.get(offset..);
        }

        let mut v = variant;
        for chunk in class_name.split('_') {
            let is_variant_section = Some('_') == v.chars().nth(chunk.len());
            if v.starts_with(chunk) && is_variant_section {
                let offset = chunk.len() + 1;
                let next = &v[offset..];
                if next.chars().next()?.is_numeric() {
                    break;
                }
                v = next;
            }
        }
        if v != variant {
            return Some(v);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_enum() {
        let actual = generate_enum_variant_name("FooBar", "BAZ");
        assert_eq!("BAZ", actual);
    }

    #[test]
    fn blend_mode() {
        let actual = generate_enum_variant_name("BlendMode", "BLEND_MODE_INTERPOLATED");
        assert_eq!("INTERPOLATED", actual);
    }

    #[test]
    fn track_type() {
        let actual = generate_enum_variant_name("TrackType", "TYPE_BEZIER");
        assert_eq!("BEZIER", actual);
    }

    #[test]
    fn viewport_msaa() {
        // cannot have enum variants starting with a number
        let actual = generate_enum_variant_name("ViewportMSAA", "VIEWPORT_MSAA_2X");
        assert_eq!("MSAA_2X", actual);
    }

    #[test]
    fn speaker_mode() {
        // cannot have enum variants starting with a number
        let actual = generate_enum_variant_name("SpeakerMode", "SPEAKER_MODE_STEREO");
        assert_eq!("STEREO", actual);
        let actual = generate_enum_variant_name("SpeakerMode", "SPEAKER_SURROUND_31");
        assert_eq!("SURROUND_31", actual);
    }

    #[test]
    fn attenuation_model() {
        let actual = generate_enum_variant_name("AttenuationModel", "ATTENUATION_INVERSE_DISTANCE");
        assert_eq!("INVERSE_DISTANCE", actual);
    }

    #[test]
    fn filter_db() {
        let actual = generate_enum_variant_name("FilterDB", "FILTER_6DB");
        assert_eq!("FILTER_6DB", actual);
    }

    #[test]
    fn saver_flags() {
        let actual = generate_enum_variant_name("SaverFlags", "FLAG_RELATIVE_PATHS");
        assert_eq!("RELATIVE_PATHS", actual);
    }

    #[test]
    fn poly_end_type() {
        let actual = generate_enum_variant_name("PolyEndType", "END_POLYGON");
        assert_eq!("POLYGON", actual);
    }

    #[test]
    fn viewport_update_mode() {
        let actual = generate_enum_variant_name("ViewportUpdateMode", "VIEWPORT_UPDATE_DISABLED");
        assert_eq!("DISABLED", actual);
    }

    #[test]
    fn viewport_clear_mode() {
        let actual = generate_enum_variant_name("ViewportClearMode", "VIEWPORT_CLEAR_ALWAYS");
        assert_eq!("ALWAYS", actual);
    }

    #[test]
    fn variant_section() {
        let actual = generate_enum_variant_name("FooBar", "FOOBY_BAZ");
        assert_eq!("FOOBY_BAZ", actual);
    }

    /*
    #[test]
    fn power_state() {
        let actual = generate_enum_variant_name("PowerState", "POWERSTATE_ON_BATTERY");
        assert_eq!("ON_BATTERY", actual);
    }
    */
}

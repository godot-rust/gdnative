use crate::api::*;
use crate::methods;
use crate::special_methods;
use crate::GeneratorResult;

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

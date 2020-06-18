#[macro_use]
extern crate serde_derive;

use quote::{format_ident, quote};

macro_rules! generated_at {
    ($output:ident) => {
        #[cfg(feature = "debug")]
        writeln!(
            $output,
            "\n\n// GENERATED at {}:{}:{}\n",
            module_path!(),
            line!(),
            column!()
        )?;
    };
}

pub mod api;
mod classes;
pub mod dependency;
mod documentation;
mod methods;
mod special_methods;

use std::collections::HashSet;
use std::io::Write;

pub use crate::api::*;
use crate::classes::*;
pub use crate::dependency::*;
use crate::documentation::*;
use crate::methods::*;
use crate::special_methods::*;

use std::io;

pub type GeneratorResult<T = ()> = Result<T, io::Error>;

pub fn generate_bindings(
    output_types_impls: &mut impl Write,
    output_trait_impls: &mut impl Write,
    output_method_table: &mut impl Write,
    ignore: Option<HashSet<String>>,
) -> GeneratorResult {
    let to_ignore = ignore.unwrap_or_default();

    let api = Api::new();

    generate_imports(output_types_impls)?;

    for class in &api.classes {
        // ignore classes that have been generated before.
        if to_ignore.contains(&class.name) {
            continue;
        }

        generate_class_bindings(
            output_types_impls,
            output_trait_impls,
            output_method_table,
            &api,
            &class,
        )?;
    }

    Ok(())
}

pub fn generate_imports(output: &mut impl Write) -> GeneratorResult {
    let expanded = quote! {
        use std::os::raw::c_char;
        use std::ptr;
        use std::mem;
    };

    generated_at!(output);
    write!(output, "{}", expanded)?;

    Ok(())
}

pub fn generate_class(
    output_types_impls: &mut impl Write,
    output_trait_impls: &mut impl Write,
    output_method_table: &mut impl Write,
    class_name: &str,
) -> GeneratorResult {
    let api = Api::new();

    let class = api.find_class(class_name);

    if let Some(mut class) = class {
        generate_class_bindings(
            output_types_impls,
            output_trait_impls,
            output_method_table,
            &api,
            &mut class,
        )?;
    }

    Ok(())
}

fn generate_class_bindings(
    output_types_impls: &mut impl Write,
    output_trait_impls: &mut impl Write,
    output_method_table: &mut impl Write,
    api: &Api,
    class: &GodotClass,
) -> GeneratorResult {
    // types and methods
    {
        generate_class_documentation(output_types_impls, &api, class)?;

        generate_class_struct(output_types_impls, class)?;

        generate_enums(output_types_impls, class)?;

        if !class.constants.is_empty() {
            generate_class_constants(output_types_impls, class)?;
        }

        generate_class_impl(output_types_impls, &api, class)?;
    }

    // traits
    {
        generate_godot_object_impl(output_trait_impls, class)?;

        generate_free_impl(output_trait_impls, &api, class)?;

        if !class.base_class.is_empty() {
            generate_deref_impl(output_trait_impls, class)?;
        }

        // RefCounted
        if class.is_refcounted() {
            generate_impl_ref_counted(output_trait_impls, class)?;
            generate_reference_clone(output_trait_impls, class)?;
            generate_drop(output_trait_impls, class)?;
        }

        // Instantiable
        if class.instantiable {
            generate_instantiable_impl(output_trait_impls, class)?;
        }
    }

    // methods and method table for classes with functions
    if class.instantiable || !class.methods.is_empty() {
        generate_method_table(output_method_table, &api, class)?;

        for method in &class.methods {
            generate_method_impl(output_method_table, class, method)?;
        }
    }

    Ok(())
}

fn rust_safe_name(name: &str) -> proc_macro2::Ident {
    match name {
        "use" => format_ident!("_use"),
        "type" => format_ident!("_type"),
        "loop" => format_ident!("_loop"),
        "in" => format_ident!("_in"),
        "override" => format_ident!("_override"),
        "where" => format_ident!("_where"),
        name => format_ident!("{}", name),
    }
}

#[cfg(feature = "debug")]
#[cfg(test)]
pub(crate) mod test_prelude {
    use super::*;
    use std::io::BufWriter;

    macro_rules! validate_and_clear_buffer {
        ($buffer:ident) => {
            $buffer.flush().unwrap();
            let content = std::str::from_utf8($buffer.get_ref()).unwrap();
            if let Err(_) = syn::parse_file(&content) {
                let mut code_file = std::env::temp_dir();
                code_file.set_file_name("bad_code.rs");
                std::fs::write(&code_file, &content).unwrap();
                panic!(
                    "Could not parse generated code. Check {}",
                    code_file.display()
                );
            }
            $buffer.get_mut().clear();
        };
    }

    #[test]
    fn sanity_test_generated_code() {
        let api = Api::new();
        let mut buffer = BufWriter::new(Vec::with_capacity(16384));
        for class in &api.classes {
            generate_class_documentation(&mut buffer, &api, &class).unwrap();
            write!(&mut buffer, "{}", quote! { struct Docs {} }).unwrap();
            validate_and_clear_buffer!(buffer);

            generate_class_struct(&mut buffer, &class).unwrap();
            validate_and_clear_buffer!(buffer);

            generate_enums(&mut buffer, &class).unwrap();
            validate_and_clear_buffer!(buffer);

            if !class.constants.is_empty() {
                generate_class_constants(&mut buffer, &class).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            generate_class_impl(&mut buffer, &api, &class).unwrap();
            validate_and_clear_buffer!(buffer);

            // traits
            generate_godot_object_impl(&mut buffer, &class).unwrap();
            validate_and_clear_buffer!(buffer);

            generate_free_impl(&mut buffer, &api, &class).unwrap();
            validate_and_clear_buffer!(buffer);

            if !class.base_class.is_empty() {
                generate_deref_impl(&mut buffer, &class).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // RefCounted
            if class.is_refcounted() {
                generate_impl_ref_counted(&mut buffer, &class).unwrap();
                validate_and_clear_buffer!(buffer);

                generate_reference_clone(&mut buffer, &class).unwrap();
                validate_and_clear_buffer!(buffer);

                generate_drop(&mut buffer, &class).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // Instantiable
            if class.instantiable {
                generate_instantiable_impl(&mut buffer, &class).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // methods and method table
            generate_method_table(&mut buffer, &api, &class).unwrap();
            validate_and_clear_buffer!(buffer);

            for method in &class.methods {
                generate_method_impl(&mut buffer, &class, method).unwrap();
                validate_and_clear_buffer!(buffer);
            }
        }
    }
}

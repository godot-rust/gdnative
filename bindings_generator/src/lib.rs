use proc_macro2::TokenStream;

use quote::{format_ident, quote};

pub mod api;
mod classes;
pub mod dependency;
mod documentation;
mod methods;
mod special_methods;

use std::collections::HashSet;

pub use crate::api::*;
use crate::classes::*;
pub use crate::dependency::*;
use crate::documentation::*;
use crate::methods::*;
use crate::special_methods::*;

use std::io;

pub type GeneratorResult<T = ()> = Result<T, io::Error>;

#[allow(clippy::implicit_hasher)]
pub fn generate_bindings(api: &Api, ignore: Option<HashSet<String>>) -> TokenStream {
    let to_ignore = ignore.unwrap_or_default();

    let imports = generate_imports();

    let classes = api.classes.iter().filter_map(|class| {
        // ignore classes that have been generated before.
        if to_ignore.contains(&class.name) {
            return None;
        }

        Some(generate_class_bindings(&api, &class))
    });

    quote! {
        #imports
        #(#classes)*
    }
}

pub fn generate_imports() -> TokenStream {
    quote! {
        use std::os::raw::c_char;
        use std::ptr;
        use std::mem;
    }
}

pub fn generate_class(api: &Api, class_name: &str) -> TokenStream {
    let class = api.find_class(class_name);

    if let Some(class) = class {
        generate_class_bindings(&api, &class)
    } else {
        Default::default()
    }
}

fn generate_class_bindings(api: &Api, class: &GodotClass) -> TokenStream {
    // types and methods
    let types_and_methods = {
        let documentation = generate_class_documentation(&api, class);

        let class_struct = generate_class_struct(class);

        let enums = generate_enums(class);

        let constants = if !class.constants.is_empty() {
            generate_class_constants(class)
        } else {
            Default::default()
        };

        let class_impl = generate_class_impl(&api, class);

        quote! {
            #documentation
            #class_struct
            #enums
            #constants
            #class_impl
        }
    };

    // traits
    let traits = {
        let object_impl = generate_godot_object_impl(class);

        let free_impl = generate_queue_free_impl(&api, class);

        let base_class = if !class.base_class.is_empty() {
            generate_deref_impl(class)
        } else {
            Default::default()
        };

        // Instantiable
        let instantiable = if class.instantiable {
            generate_instantiable_impl(class)
        } else {
            Default::default()
        };

        quote! {
            #object_impl
            #free_impl
            #base_class
            #instantiable
        }
    };

    // methods and method table for classes with functions
    let methods_and_table = if class.instantiable || !class.methods.is_empty() {
        let table = generate_method_table(&api, class);

        let methods = class
            .methods
            .iter()
            .map(|method| generate_method_impl(class, method));

        quote! {
            #table
            #(#methods)*
        }
    } else {
        Default::default()
    };

    quote! {
        #types_and_methods
        #traits
        #methods_and_table
    }
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
    use std::io::{BufWriter, Write};

    macro_rules! validate_and_clear_buffer {
        ($buffer:ident) => {
            $buffer.flush().unwrap();
            let content = std::str::from_utf8($buffer.get_ref()).unwrap();
            if syn::parse_file(&content).is_err() {
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
            let code = generate_class_documentation(&api, &class);
            write!(&mut buffer, "{}", code).unwrap();
            write!(&mut buffer, "{}", quote! { struct Docs {} }).unwrap();
            validate_and_clear_buffer!(buffer);

            let code = generate_class_struct(&class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            let code = generate_enums(&class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            if !class.constants.is_empty() {
                let code = generate_class_constants(&class);
                write!(&mut buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            let code = generate_class_impl(&api, &class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            // traits
            let code = generate_godot_object_impl(&class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            let code = generate_queue_free_impl(&api, &class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            if !class.base_class.is_empty() {
                let code = generate_deref_impl(&class);
                write!(&mut buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // Instantiable
            if class.instantiable {
                let code = generate_instantiable_impl(&class);
                write!(&mut buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // methods and method table
            let code = generate_method_table(&api, &class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            for method in &class.methods {
                let code = generate_method_impl(&class, method);
                write!(&mut buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }
        }
    }
}

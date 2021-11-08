//! Internal API bindings generator for the godot-rust bindings.
//!
//! # Creating custom binding crates
//!
//! It's possible to create custom binding crates without forking the repository by passing
//! custom `api.json` data to `Api::new()`. The JSON data can be generated from Godot using
//! the following command:
//!
//! `/path/to/godot --gdnative-generate-json-api /path/to/api.json`
//!
//! *Please note that The generator is an internal dependency.* As such, it is not covered
//! by semver guarantees of the main `gdnative` crate. When using custom binding crates, care
//! must be taken to ensure that the version of the generator matches the one specified in
//! the `Cargo.toml` of the `gdnative` crate exactly, even for updates that are considered
//! non-breaking in the `gdnative` crate.
use proc_macro2::TokenStream;

use quote::{format_ident, quote};

pub mod api;
mod class_docs;
mod classes;
pub mod dependency;
mod documentation;
mod methods;
mod special_methods;

pub use crate::api::*;
pub use crate::class_docs::*;
use crate::classes::*;
pub use crate::dependency::*;
use crate::documentation::*;
use crate::methods::*;
use crate::special_methods::*;

use std::collections::HashMap;
use std::io;

pub type GeneratorResult<T = ()> = Result<T, io::Error>;

pub struct BindingResult {
    pub class_bindings: HashMap<String, TokenStream>,
    pub icalls: TokenStream,
}

pub fn generate_bindings(api: &Api, docs: Option<&GodotXmlDocs>) -> BindingResult {
    let mut icalls = HashMap::new();

    let class_bindings = api
        .classes
        .iter()
        .map(|class| {
            (
                class.name.clone(),
                generate_class_bindings(api, class, &mut icalls, docs),
            )
        })
        .collect();

    let icalls = icalls
        .into_iter()
        .map(|(name, sig)| generate_icall(name, sig))
        .collect();

    BindingResult {
        class_bindings,
        icalls,
    }
}

pub fn generate_imports() -> TokenStream {
    quote! {
        use std::os::raw::c_char;
        use std::ptr;
        use std::mem;
    }
}

fn generate_class_bindings(
    api: &Api,
    class: &GodotClass,
    icalls: &mut HashMap<String, MethodSig>,
    docs: Option<&GodotXmlDocs>,
) -> TokenStream {
    // types and methods
    let types_and_methods = {
        let documentation = generate_class_documentation(api, class);

        let class_struct = generate_class_struct(class);

        let enums = generate_enums(class);

        let constants = if !class.constants.is_empty() {
            generate_class_constants(class)
        } else {
            Default::default()
        };

        let class_impl = generate_class_impl(class, icalls, docs);

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

        let free_impl = generate_queue_free_impl(api, class);

        let base_class = if !class.base_class.is_empty() {
            generate_deref_impl(class)
        } else {
            Default::default()
        };

        let sub_class = generate_sub_class_impls(api, class);

        // Instantiable
        let instantiable = if class.instantiable {
            generate_instantiable_impl(class)
        } else {
            Default::default()
        };

        let send_sync = if class.singleton && class.is_singleton_thread_safe() {
            generate_send_sync_impls(class)
        } else {
            Default::default()
        };

        quote! {
            #object_impl
            #free_impl
            #base_class
            #sub_class
            #instantiable
            #send_sync
        }
    };

    // method table for classes with functions
    let method_table = if class.instantiable || !class.methods.is_empty() {
        generate_method_table(api, class)
    } else {
        Default::default()
    };

    quote! {
        #types_and_methods
        #traits
        #method_table
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
        "enum" => format_ident!("_enum"),
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
        let api = Api::new(include_str!("../../gdnative-bindings/api.json"));
        let mut buffer = BufWriter::new(Vec::with_capacity(16384));
        for class in &api.classes {
            let mut icalls = HashMap::new();

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

            let code = generate_class_impl(&class, &mut icalls, None);
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

            // icalls and method table
            let code = generate_method_table(&api, &class);
            write!(&mut buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            for (name, sig) in icalls {
                let code = generate_icall(name, sig);
                write!(&mut buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }
        }
    }
}

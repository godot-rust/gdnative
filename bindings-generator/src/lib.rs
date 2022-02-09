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

mod class_docs;
mod classes;
mod documentation;
mod methods;
mod special_methods;

#[cfg(feature = "custom-godot")]
mod godot_api_json;
mod godot_version;

pub mod api;
pub mod dependency;

use crate::classes::*;
use crate::documentation::*;
use crate::methods::*;
use crate::special_methods::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::io;

pub use api::*;
pub use class_docs::*;
pub use dependency::*;

#[cfg(feature = "custom-godot")]
pub use godot_api_json::*;
pub use godot_version::*;

#[cfg(not(feature = "custom-godot"))]
pub fn generate_json_if_needed() -> bool {
    false
}

pub type GeneratorResult<T = ()> = Result<T, io::Error>;

pub struct BindingResult<'a> {
    pub class_bindings: Vec<(&'a GodotClass, TokenStream)>,
    pub icalls: TokenStream,
}

pub fn generate_bindings<'a>(api: &'a Api, docs: Option<&GodotXmlDocs>) -> BindingResult<'a> {
    let mut icalls = HashMap::new();

    let class_bindings = api
        .classes
        .iter()
        .map(|class| {
            (
                class,
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
        let module_doc = generate_module_doc(class);
        let class_doc = generate_class_documentation(api, class);
        let class_struct = generate_class_struct(class, class_doc);

        let enums = generate_enums(class);

        let constants = if !class.constants.is_empty() {
            generate_class_constants(class)
        } else {
            Default::default()
        };

        let class_impl = generate_class_impl(class, icalls, docs);

        quote! {
            #module_doc
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

#[rustfmt::skip]
fn rust_safe_name(name: &str) -> proc_macro2::Ident {
    // Keywords obtained from https://doc.rust-lang.org/reference/keywords.html
    match name {
        // Lexer 2015
        "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false" | "fn" | "for" |
        "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" |
        "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" | "type" | "unsafe" | "use" |
        "where" | "while" |
        
        // Lexer 2018
        "async" | "await" | "dyn" |
        
        // Lexer 2018+
        "try" |
        
        // Reserved words
        "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "typeof" |
        "unsized" | "virtual" | "yield"
          => format_ident!("{}_", name),

        _ => format_ident!("{}", name)
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
        // Tests whether each generated snippet individually constitutes a valid AST representation of Rust code

        let api = Api::new(include_str!("../../gdnative-bindings/api.json"));
        let mut buffer = BufWriter::new(Vec::with_capacity(16384));
        for class in &api.classes {
            let mut icalls = HashMap::new();

            let code = generate_module_doc(&class);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            let class_doc = generate_class_documentation(&api, &class);
            write!(buffer, "{}", code).unwrap();
            write!(buffer, "{}", quote! { struct StructWithDocs {} }).unwrap();
            validate_and_clear_buffer!(buffer);

            let code = generate_class_struct(&class, class_doc);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            let code = generate_enums(&class);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            if !class.constants.is_empty() {
                let code = generate_class_constants(&class);
                write!(buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            let code = generate_class_impl(&class, &mut icalls, None);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            // traits
            let code = generate_godot_object_impl(&class);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            let code = generate_queue_free_impl(&api, &class);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            if !class.base_class.is_empty() {
                let code = generate_deref_impl(&class);
                write!(buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // Instantiable
            if class.instantiable {
                let code = generate_instantiable_impl(&class);
                write!(buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }

            // icalls and method table
            let code = generate_method_table(&api, &class);
            write!(buffer, "{}", code).unwrap();
            validate_and_clear_buffer!(buffer);

            for (name, sig) in icalls {
                let code = generate_icall(name, sig);
                write!(buffer, "{}", code).unwrap();
                validate_and_clear_buffer!(buffer);
            }
        }
    }
}

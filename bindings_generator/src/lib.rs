
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate heck;

pub mod json;
mod classes;
mod methods;
mod special_methods;
mod documentation;

use std::fs::File;
use std::io::Write;
use std::collections::HashSet;

pub use json::*;
use classes::*;
use methods::*;
use special_methods::*;
use documentation::*;

use std::io;

pub type GeneratorResult = Result<(), io::Error>;

pub fn generate_bindings(
    api_description: File,
    api_namespaces: File,
    output: &mut File,
    crate_type: Crate,
) -> GeneratorResult {

    let api = Api::new(api_description, api_namespaces, crate_type);

    writeln!(output, "use std::os::raw::c_char;")?;
    writeln!(output, "use std::ptr;")?;
    writeln!(output, "use std::mem;")?;

    for class in &api.classes {
        if api.namespaces[&class.name] != crate_type {
            continue;
        }

        generate_class_documentation(output, &api, class)?;

        generate_class_struct(output, class)?;

        for e in &class.enums {
            generate_enum(output, class, e)?;
        }

        generate_godot_object_impl(output, class)?;

        generate_free_impl(output, &api, class)?;

        writeln!(output, "impl {} {{", class.name)?;

        if class.singleton {
            generate_singleton_getter(output, class)?;
        }

        if class.instanciable {

            if class.is_refcounted() {
                generate_refreference_ctor(output, class)?;
            } else {
                generate_non_refreference_ctor(output, class)?;
            }
        }

        let mut method_set = HashSet::default();

        generate_methods(
            output,
            &api,
            &mut method_set,
            &class.name,
            class.is_pointer_safe(),
            true,
        )?;

        generate_upcast(
            output,
            &api,
            &class.base_class,
            class.is_pointer_safe(),
        )?;

        generate_dynamic_cast(output, class)?;

        writeln!(output, "}}")?;

        if class.is_refcounted() && class.instanciable {
            generate_drop(output, class)?;
        }
    }

//    writeln!(output,
//r#"#[doc(hidden)]
//pub mod gdnative_{:?}_private {{
//
//use std::sync::{{Once, ONCE_INIT}};
//use std::os::raw::c_char;
//use std::ptr;
//use std::mem;
//use libc;
//use object;"#,
//        api.sub_crate
//    ).unwrap();
//
//    if api.sub_crate != Crate::core {
//        writeln!(output, "use gdnative_core::*;").unwrap();
//    } else{
//        writeln!(output, "use super::*;").unwrap();
//    }

    for class in &api.classes {
        if api.namespaces[&class.name] != crate_type {
            continue;
        }

        generate_method_table(output, class)?;

        for method in &class.methods {
            generate_method_impl(output, class, method)?;
        }
    }

//    writeln!(output, "\n}} // private module").unwrap();

    Ok(())
}

fn rust_safe_name(name: &str) -> &str {
    match name {
        "use" => "_use",
        "type" => "_type",
        "loop" => "_loop",
        "in" => "_in",
        "override" => "_override",
        "where" => "_where",
        name => name,
    }
}

pub fn get_crate_namespace_opt(crate_type: Option<Crate>) -> &'static str {
    match crate_type {
        Some(ty) => get_crate_namespace(ty),
        None => ""
    }
}

pub fn get_crate_namespace(crate_type: Crate) -> &'static str {
    match crate_type {
        Crate::core => "core",
        Crate::common => "common",
        Crate::graphics => "graphics",
        Crate::animation => "animation",
        Crate::physics => "physics",
        Crate::network => "network",
        Crate::audio => "audio",
        Crate::video => "video",
        Crate::arvr => "arvr",
        Crate::input => "input",
        Crate::ui => "ui",
        Crate::editor => "editor",
        Crate::visual_script => "visual_script",
        Crate::unknown => "unknown",
    }
}


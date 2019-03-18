#[macro_use]
extern crate serde_derive;

pub mod api;
pub mod dependency;
mod classes;
mod methods;
mod special_methods;
mod documentation;

use std::fs::File;
use std::io::Write;
use std::collections::HashSet;

pub use crate::api::*;
pub use crate::dependency::*;
use crate::classes::*;
use crate::methods::*;
use crate::special_methods::*;
use crate::documentation::*;

use std::io;

pub type GeneratorResult<T = ()> = Result<T, io::Error>;

pub fn generate_bindings(
    output: &mut File,
    ignore: Option<HashSet<String>>,
) -> GeneratorResult {

    let to_ignore = ignore.unwrap_or_default();

    let api = Api::new();

    generate_imports(output)?;

    for class in &api.classes {

        // ignore classes that have been generated before.
        if to_ignore.contains(&class.name) {
            continue;
        }

        generate_class_bindings(output, &api, class)?;

    }

    Ok(())
}

pub fn generate_imports(output: &mut impl Write) -> GeneratorResult {
    writeln!(output, "use std::os::raw::c_char;")?;
    writeln!(output, "use std::ptr;")?;
    writeln!(output, "use std::mem;")?;

    Ok(())
}

pub fn generate_class(
    output: &mut impl io::Write,
    class_name: &str,
) -> GeneratorResult {

    let api = Api::new();

    let class = api.find_class(class_name);

    if let Some(class) = class {
        generate_class_bindings(output, &api, class)?;
    }

    Ok(())
}

fn generate_class_bindings(
    output: &mut impl io::Write,
    api: &Api,
    class: &GodotClass,
) -> GeneratorResult {

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

    generate_method_table(output, class)?;

    for method in &class.methods {
        generate_method_impl(output, class, method)?;
    }

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


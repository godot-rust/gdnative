#[macro_use]
extern crate serde_derive;

pub mod api;
pub mod dependency;
mod classes;
mod methods;
mod special_methods;
mod documentation;

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
            class,
        )?;

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
    output_types_impls: &mut impl Write,
    output_trait_impls: &mut impl Write,
    output_method_table: &mut impl Write,
    class_name: &str,
) -> GeneratorResult {

    let api = Api::new();

    let class = api.find_class(class_name);

    if let Some(class) = class {
        generate_class_bindings(
            output_types_impls,
            output_trait_impls,
            output_method_table,
            &api,
            class,
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

        for e in &class.enums {
            generate_enum(output_types_impls, class, e)?;
        }

        writeln!(output_types_impls, "impl {} {{", class.name)?;

        if class.singleton {
            generate_singleton_getter(output_types_impls, class)?;
        }

        if class.instanciable {
            if class.is_refcounted() {
                generate_reference_ctor(output_types_impls, class)?;
            } else {
                generate_non_reference_ctor(output_types_impls, class)?;
            }
        }

        if class.is_refcounted() {
            generate_reference_copy(output_types_impls, class)?;
        }

        let mut method_set = HashSet::default();

        generate_methods(
            output_types_impls,
            &api,
            &mut method_set,
            &class.name,
            class.is_pointer_safe(),
            true,
        )?;

        generate_upcast(
            output_types_impls,
            &api,
            &class.base_class,
            class.is_pointer_safe(),
        )?;

        generate_dynamic_cast(output_types_impls, class)?;

        writeln!(output_types_impls, "}}")?;
    }

    // traits
    {
        generate_godot_object_impl(output_trait_impls, class)?;

        generate_free_impl(output_trait_impls, &api, class)?;


        if !class.base_class.is_empty() {
            generate_deref_impl(output_trait_impls, class)?;
        }

        if class.is_refcounted() {
            generate_reference_clone(output_trait_impls, class)?;
        }

        if class.is_refcounted() && class.instanciable {
            generate_drop(output_trait_impls, class)?;
        }
    }

    // methods and method table
    {
        let has_underscore = api.api_underscore.iter().any(|name| class.name.starts_with(name));
        generate_method_table(output_method_table, class, has_underscore)?;

        for method in &class.methods {
            generate_method_impl(output_method_table, class, method)?;
        }
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


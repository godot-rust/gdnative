
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate heck;

mod json;
mod classes;
mod methods;
mod special_methods;
mod documentation;

use std::fs::File;
use std::io::Write;
use std::collections::HashSet;

use json::*;
use classes::*;
use methods::*;
use special_methods::*;
use documentation::*;

pub fn generate_bindings(api_description: File, output: &mut File) {

    let classes: Vec<GodotClass> = serde_json::from_reader(api_description)
        .expect("Failed to parse the API description");

    writeln!(output, "use std::os::raw::c_char;").unwrap();
    writeln!(output, "use std::ptr;").unwrap();
    writeln!(output, "use std::mem;").unwrap();
    writeln!(output, "use object;").unwrap();

    for class in &classes {
        generate_class_documentation(output, &classes, class);

        generate_class_struct(output, class);

        for e in &class.enums {
            generate_enum(output, class, e);
        }

        generate_method_table(output, class);

        generate_godot_object_impl(output, class);

        for method in &class.methods {
            generate_method_impl(output, class, method);
        }

        writeln!(output, "impl {} {{", class.name).unwrap();

        if class.singleton {
            generate_singleton_getter(output, class);
        }

        if class.instanciable {

            if class.is_refcounted() {
                generate_refreference_ctor(output, class);
            } else {
                generate_non_refreference_ctor(output, class);
            }
        }

        let mut method_set = HashSet::default();

        generate_methods(
            output,
            &classes,
            &mut method_set,
            &class.name,
            class.is_pointer_safe(),
        );

        generate_upcast(
            output,
            &classes,
            &class.base_class,
            class.is_pointer_safe(),
        );

        generate_dynamic_cast(output, class);

        writeln!(output, "}}").unwrap();

        if class.is_refcounted() && class.instanciable {
            generate_drop(output, class);
        }
    }
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

pub fn find_class<'a, 'b>(classes: &'a[GodotClass], name: &'b str) -> Option<&'a GodotClass> {
    for class in classes {
        if &class.name == name {
            return Some(class);
        }
    }

    None
}

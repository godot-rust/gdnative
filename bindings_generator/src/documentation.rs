use json::*;
use std::fs::File;
use std::io::Write;

use find_class;

pub fn generate_class_documentation(output: &mut File, classes: &[GodotClass], class: &GodotClass) {
        let has_parent = class.base_class != "";
        let singleton_str = if class.singleton { "singleton " } else { "" } ;
        let ownership_type = if class.is_refcounted() { "reference counted" } else { "manually managed" };
        if &class.name == "Reference" {
            writeln!(output, "/// Base class of all reference-counted types. Inherits `Object`.").unwrap();
        } else if &class.name == "Object" {
            writeln!(output, "/// The base class of most Godot classes.").unwrap();
        } else if has_parent {
            writeln!(output, r#"
/// `{api_type} {singleton}class {name}` inherits `{base_class}` ({ownership_type})."#,
                api_type = class.api_type,
                name = class.name,
                base_class = class.base_class,
                ownership_type = ownership_type,
                singleton = singleton_str
            ).unwrap();
        } else {
            writeln!(output, r#"
/// `{api_type} {singleton}class {name}` ({ownership_type})."#,
                api_type = class.api_type,
                name = class.name,
                ownership_type = ownership_type,
                singleton = singleton_str
            ).unwrap();
        }

        if class.is_refcounted() {
            writeln!(output,
r#"///
/// ## Memory management
///
/// The lifetime of this object is automatically managed through reference counting."#
            ).unwrap();
        } else if class.instanciable {
            writeln!(output,
r#"///
/// ## Memory management
///
/// Non reference counted objects such as the ones of this type are usually owned by the engine.
///
/// `{name}` is an unsafe pointer, and all of its methods are unsafe.
///
/// In the cases where Rust code owns an object of this type, for example if the object was just
/// created on the Rust side and not passed to the engine yet, ownership should be either given
/// to the engine or the object must be manually destroyed using `{name}::free`."#,
                name = class.name
            ).unwrap();
        }

        if class.base_class != "" {
            writeln!(output,
r#"///
/// ## Class hierarchy
///
/// {name} inherits methods from:"#,
                name = class.name,
            ).unwrap();

            list_base_classes(
                output,
                &classes,
                &class.base_class,
            );
        }

        if class.api_type == "tools" {
            writeln!(output,
r#"///
/// ## Tool
///
/// This class is used to interact with Godot's editor."#,
            ).unwrap();
        }
}

fn list_base_classes(
    output: &mut File,
    classes: &[GodotClass],
    parent_name: &str,
) {
    if let Some(parent) = find_class(classes, parent_name) {
        writeln!(output,
            "/// - [{base_class}](struct.{base_class}.html)",
            base_class = parent_name,
        ).unwrap();

        if parent.base_class != "" {
            list_base_classes(output, classes, &parent.base_class);
        }
    }
}

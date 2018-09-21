use json::*;
use std::fs::File;
use std::io::Write;
use GeneratorResult;

pub fn class_doc_link(class: &GodotClass) -> String {
    // TODO: link the correct crate
    // let subcrate = get_crate(class);
    format!("[{name}](struct.{name}.html)", name = class.name)
}

pub fn official_doc_url(class: &GodotClass) -> String {
    format!(
        "https://godot.readthedocs.io/en/3.0/classes/class_{lower_case}.html",
        lower_case = class.name.to_lowercase(),
    )
}

pub fn generate_class_documentation(output: &mut File, api: &Api, class: &GodotClass) -> GeneratorResult {
        let has_parent = class.base_class != "";
        let singleton_str = if class.singleton { "singleton " } else { "" } ;
        let ownership_type = if class.is_refcounted() { "reference counted" } else { "manually managed" };
        if &class.name == "Reference" {
            writeln!(output, "/// Base class of all reference-counted types. Inherits `Object`.")?;
        } else if &class.name == "Object" {
            writeln!(output, "/// The base class of most Godot classes.")?;
        } else if has_parent {
            writeln!(output, r#"
/// `{api_type} {singleton}class {name}` inherits `{base_class}` ({ownership_type})."#,
                api_type = class.api_type,
                name = class.name,
                base_class = class.base_class,
                ownership_type = ownership_type,
                singleton = singleton_str
            )?;
        } else {
            writeln!(output, r#"
/// `{api_type} {singleton}class {name}` ({ownership_type})."#,
                api_type = class.api_type,
                name = class.name,
                ownership_type = ownership_type,
                singleton = singleton_str
            )?;
        }

        writeln!(output,
r#"///
/// ## Official documentation
///
/// See the [documentation of this class]({url}) in the Godot engine's official documentation."#,
            url = official_doc_url(class),
        )?;

        if class.is_refcounted() {
            writeln!(output,
r#"///
/// ## Memory management
///
/// The lifetime of this object is automatically managed through reference counting."#
            )?;
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
            )?;
        }

        if class.base_class != "" {
            writeln!(output,
r#"///
/// ## Class hierarchy
///
/// {name} inherits methods from:"#,
                name = class.name,
            )?;

            list_base_classes(
                output,
                api,
                &class.base_class,
            )?;
        }

        if class.api_type == "tools" {
            writeln!(output,
r#"///
/// ## Tool
///
/// This class is used to interact with Godot's editor."#,
            )?;
        }

        Ok(())
}

fn list_base_classes(
    output: &mut File,
    api: &Api,
    parent_name: &str,
) -> GeneratorResult {
    if let Some(parent) = api.find_class(parent_name) {
        let class_link = class_doc_link(&parent);

        writeln!(output, "/// - {}", class_link)?;

        if parent.base_class != "" {
            list_base_classes(output, api, &parent.base_class)?;
        }
    }

    Ok(())
}

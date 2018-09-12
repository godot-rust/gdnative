extern crate gdnative_bindings_generator;
extern crate serde;
extern crate serde_json;

use gdnative_bindings_generator::*;
use gdnative_bindings_generator::json::*;
use std::env;
use std::fs::File;
use std::collections::{HashMap, HashSet};
use Crate;

fn main() {
    let api_path = env::args().nth(1).unwrap();
    let namespace_path = env::args().nth(2).unwrap();
    let class_name = env::args().nth(3).unwrap();

    print_dependencies(
        File::open(&api_path).unwrap(),
        File::open(&namespace_path).unwrap(),
        &class_name,
    );
}

pub fn print_dependencies(
    api_description: File,
    api_namespaces: File,
    class_name: &str,
) {
    let classes: Vec<GodotClass> = serde_json::from_reader(api_description).expect("Failed to parse the API description");
    let namespaces: HashMap<String, Crate> = serde_json::from_reader(api_namespaces).expect("Failed to parse the API namespaces");

    if let Some(class) = find_class(&classes, class_name) {
        println!("class {} ({:?})", class_name, namespaces[class_name]);
        println!("Depends on:");
        let mut deps: HashSet<String> = HashSet::default();
        if class.base_class != "" {
            deps.insert(class.base_class.clone());
        }
        for method in &class.methods {
            for arg in &method.arguments {
                deps.insert(arg.ty.clone());
            }
            deps.insert(method.return_type.clone());
        }

        let mut deps: Vec<String> = deps.into_iter().collect();
        deps.sort();
        for dep in deps {
            if namespaces.contains_key(&dep) {
                println!(" - {} ({:?})", dep, namespaces[&dep]);
            } else {
                println!(" - {}", dep);
            }
        }

        println!("Is a dependency of:");
        'class_loop:
        for class in &classes {
            if class.name == class_name {
                continue;
            }

            if class.base_class == class_name {
                if namespaces.contains_key(&class.name) {
                    println!(" - {} ({:?})", class.name, namespaces[&class.name]);
                } else {
                    println!(" - {}", class.name);
                }
                continue 'class_loop;
            }

            for method in &class.methods {
                for arg in &method.arguments {
                    if arg.ty == class_name {
                        if namespaces.contains_key(&class.name) {
                            println!(" - {} ({:?})", class.name, namespaces[&class.name]);
                        } else {
                            println!(" - {}", class.name);
                        }
                        continue 'class_loop;
                    }
                }
                if method.return_type == class_name {
                    if namespaces.contains_key(&class.name) {
                        println!(" - {} ({:?})", class.name, namespaces[&class.name]);
                    } else {
                        println!(" - {}", class.name);
                    }
                    continue 'class_loop;
                }
            }
        }
    }
}

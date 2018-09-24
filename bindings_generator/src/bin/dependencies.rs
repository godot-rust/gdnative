extern crate gdnative_bindings_generator;

use gdnative_bindings_generator::*;
use std::env;
use std::fs::File;
use std::collections::HashSet;

fn main() {
    let class_name = env::args().nth(1).unwrap();
    print_dependencies(&class_name);
}

pub fn print_dependencies(class_name: &str) {
    let api = Api::new(Crate::unknown);

    if let Some(class) = api.find_class(class_name) {
        println!("class {} ({:?})", class_name, api.namespaces[class_name]);
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
            if api.namespaces.contains_key(&dep) {
                println!(" - {} ({:?})", dep, api.namespaces[&dep]);
            } else {
                println!(" - {}", dep);
            }
        }

        println!("Is a dependency of:");
        'class_loop:
        for class in &api.classes {
            if class.name == class_name {
                continue;
            }

            if class.base_class == class_name {
                if api.namespaces.contains_key(&class.name) {
                    println!(" - {} ({:?})", class.name, api.namespaces[&class.name]);
                } else {
                    println!(" - {}", class.name);
                }
                continue 'class_loop;
            }

            for method in &class.methods {
                for arg in &method.arguments {
                    if arg.ty == class_name {
                        if api.namespaces.contains_key(&class.name) {
                            println!(" - {} ({:?})", class.name, api.namespaces[&class.name]);
                        } else {
                            println!(" - {}", class.name);
                        }
                        continue 'class_loop;
                    }
                }
                if method.return_type == class_name {
                    if api.namespaces.contains_key(&class.name) {
                        println!(" - {} ({:?})", class.name, api.namespaces[&class.name]);
                    } else {
                        println!(" - {}", class.name);
                    }
                    continue 'class_loop;
                }
            }
        }
    }
}

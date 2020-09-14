use std::{collections::HashMap, fs};

use roxmltree::Node;

#[derive(Debug)]
pub struct GodotXMLDocs {
    class_fn_desc: HashMap<(String, String), String>,
}

impl GodotXMLDocs {
    pub fn new(folder: &str) -> Self {
        let entries = fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();

        let mut docs = GodotXMLDocs {
            class_fn_desc: HashMap::default(),
        };

        for entry in entries {
            if entry.extension().map(|ext| ext == "xml").is_some() {
                let file_content =
                    std::fs::read_to_string(entry.as_os_str()).expect("Unable to read file");
                docs.parse_file(file_content.as_str());
            }
        }

        docs
    }

    pub fn get_class_method_desc(&self, class: &str, method: &str) -> Option<&str> {
        let key = (class.to_string(), method.to_string());
        self.class_fn_desc.get(&key).map(|s| s.as_str())
    }

    fn parse_file(&mut self, file_content: &str) {
        let doc = roxmltree::Document::parse(file_content).unwrap();

        if let Some(class) = doc
            .descendants()
            .find(|node| node.tag_name().name() == "class")
        {
            if let Some(class_name) = class.attribute("name") {
                let members_node = class
                    .descendants()
                    .find(|node| node.tag_name().name() == "members");
                self.parse_members(class_name, members_node);

                let methods_node = class
                    .descendants()
                    .find(|node| node.tag_name().name() == "methods");
                self.parse_methods(class_name, methods_node);
            }
        }
    }

    fn parse_members(&mut self, class: &str, members: Option<Node>) {
        if let Some(members) = members {
            for node in members.descendants() {
                if node.tag_name().name() == "member" {
                    if let Some(desc) = node.text() {
                        if let Some(func) = node.attribute("setter") {
                            self.add_fn(class, func, desc);
                        }
                        if let Some(func) = node.attribute("getter") {
                            self.add_fn(class, func, desc);
                        }
                    }
                }
            }
        }
    }

    fn parse_methods(&mut self, class: &str, methods: Option<Node>) {
        if let Some(methods) = methods {
            for node in methods.descendants() {
                if node.tag_name().name() == "method" {
                    self.parse_method(class, node);
                }
            }
        }
    }

    fn parse_method(&mut self, class: &str, method: Node) {
        if let Some(method_name) = method.attribute("name") {
            if let Some(desc_node) = method
                .descendants()
                .find(|node| node.tag_name().name() == "description")
            {
                if let Some(desc) = desc_node.text() {
                    self.add_fn(class, method_name, desc);
                }
            }
        }
    }

    fn add_fn(&mut self, class: &str, method: &str, desc: &str) {
        self.class_fn_desc
            .insert((class.into(), method.into()), desc.trim().into());
    }
}

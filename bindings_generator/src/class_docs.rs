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
                let methods_node = class
                    .descendants()
                    .find(|node| node.tag_name().name() == "methods");
                self.parse_methods(class_name, methods_node);

                let members_node = class
                    .descendants()
                    .find(|node| node.tag_name().name() == "members");
                self.parse_members(class_name, members_node);
            }
        }
    }

    fn parse_members(&mut self, class: &str, members: Option<Node>) {
        if let Some(members) = members {
            for node in members.descendants() {
                if node.tag_name().name() == "member" {
                    if let Some(desc) = node.text() {
                        if let Some(func) = node.attribute("setter") {
                            self.add_fn(class, func, desc, &[]);
                        }
                        if let Some(func) = node.attribute("getter") {
                            self.add_fn(class, func, desc, &[]);
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
            let default_args = method
                .descendants()
                .filter_map(|node| {
                    if node.tag_name().name() == "argument" {
                        let key = node
                            .attribute("name")
                            .expect("expecting argument tags to have name attribute");
                        node.attribute("default").map(|val| (key, val))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(&str, &str)>>();

            if let Some(desc_node) = method
                .descendants()
                .find(|node| node.tag_name().name() == "description")
            {
                if let Some(desc) = desc_node.text() {
                    self.add_fn(class, method_name, desc, default_args.as_slice());
                }
            }
        }
    }

    fn add_fn(&mut self, class: &str, method: &str, desc: &str, default_args: &[(&str, &str)]) {
        let mut doc = unindent::unindent(desc.trim());

        if doc.is_empty() && default_args.is_empty() {
            return;
        }

        if !default_args.is_empty() {
            doc.push_str("\n# Default Arguments");

            for arg in default_args {
                doc.push_str(format!("\n* `{}` - `{}`", arg.0, arg.1).as_str());
            }
        }

        self.class_fn_desc
            .insert((class.into(), method.into()), Self::reformat_as_rustdoc(doc));
    }

    /// Takes the Godot documentation markup and transforms it to Rustdoc.
    /// Very basic approach with limitations, but already helps readability quite a bit.
    fn reformat_as_rustdoc(godot_doc: String) -> String {
        godot_doc
            .replace("[code]", "`")
            .replace("[/code]", "`")
            .replace("[codeblock]", "```gdscript")
            .replace("[/codeblock]", "```")
            .replace("[b]", "**")
            .replace("[/b]", "**")
    }
}

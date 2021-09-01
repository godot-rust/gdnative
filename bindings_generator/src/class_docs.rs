use std::{collections::HashMap, fs};

use regex::{Captures, Regex};
use roxmltree::Node;

#[derive(Debug)]
pub struct GodotXmlDocs {
    class_fn_desc: HashMap<(String, String), String>,
}

impl GodotXmlDocs {
    pub fn new(folder: &str) -> Self {
        let entries = fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();

        let mut docs = GodotXmlDocs {
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

        self.class_fn_desc.insert(
            (class.into(), method.into()),
            Self::reformat_as_rustdoc(doc),
        );
    }

    fn to_rust_type(godot_type: &str) -> &str {
        match godot_type {
            "String" => "GodotString",
            "Error" => "GodotError",
            "RID" => "Rid",
            "G6DOFJointAxisParam" => "G6dofJointAxisParam",
            "G6DOFJointAxisFlag" => "G6dofJointAxisFlag",
            _ => godot_type,
        }
    }

    /// Takes the Godot documentation markup and transforms it to Rustdoc.
    /// Very basic approach with limitations, but already helps readability quite a bit.
    fn reformat_as_rustdoc(godot_doc: String) -> String {
        let gdscript_note = if godot_doc.contains("[codeblock]") {
            "_Sample code is GDScript unless otherwise noted._\n\n"
        } else {
            ""
        };

        // TODO reuse regex across classes
        let url_regex = Regex::new("\\[url=(.+?)](.*?)\\[/url]").unwrap();

        let type_regex = Regex::new("\\[enum ([A-Za-z0-9_]+?)]").unwrap();
        let self_member_regex =
            Regex::new("\\[(member|method|constant) ([A-Za-z0-9_]+?)]").unwrap();
        let class_member_regex =
            Regex::new("\\[(member|method|constant) ([A-Za-z0-9_]+?)\\.([A-Za-z0-9_]+?)]").unwrap();

        // URLs
        let godot_doc = url_regex.replace_all(&godot_doc, |c: &Captures| {
            let url = &c[1];
            let text = &c[2];

            if text.is_empty() {
                format!("<{url}>", url = url)
            } else {
                format!("[{text}]({url})", text = text, url = url)
            }
        });

        // Note: we currently don't use c[1], which would be the "kind" (method/member/constant/...)
        // This one could be used to disambiguate the doc-link, e.g. [`{method}`][fn@Self::{method}]

        // What currently doesn't work are "indexed properties" which are not also exposed as getters, e.g.
        // https://docs.godotengine.org/en/stable/classes/class_area2d.html#properties 'gravity_point'
        // This needs to be implemented first: https://github.com/godot-rust/godot-rust/issues/689

        // TODO: [signal M]

        // [Type] style
        let godot_doc = type_regex.replace_all(&godot_doc, |c: &Captures| {
            let godot_ty = &c[1];
            let rust_ty = Self::to_rust_type(godot_ty);

            format!(
                "[`{godot_ty}`][{rust_ty}]",
                godot_ty = godot_ty,
                rust_ty = rust_ty
            )
        });

        // [Type::member] style
        let godot_doc = class_member_regex.replace_all(&godot_doc, |c: &Captures| {
            let godot_ty = &c[2];
            let rust_ty = Self::to_rust_type(godot_ty);

            format!(
                "[`{godot_ty}.{member}`][{rust_ty}::{member}]",
                godot_ty = godot_ty,
                rust_ty = rust_ty,
                member = &c[3]
            )
        });

        // [member] style
        let godot_doc = self_member_regex.replace_all(&godot_doc, |c: &Captures| {
            format!("[`{member}`][Self::{member}]", member = &c[2])
        });

        let translated = godot_doc
            .replace("[code]", "`")
            .replace("[/code]", "`")
            .replace("[codeblock]", "```gdscript")
            .replace("[/codeblock]", "```")
            .replace("[b]", "**")
            .replace("[/b]", "**");

        format!("{}{}", gdscript_note, translated)
    }
}

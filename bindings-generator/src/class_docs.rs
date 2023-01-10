use std::{collections::HashMap, fs};

use regex::{Captures, Regex};
use roxmltree::Node;

pub struct GodotXmlDocs {
    class_fn_desc: HashMap<(String, String), String>,
    regexes: Regexes,
}

impl GodotXmlDocs {
    pub fn new(folder: &str) -> Self {
        let entries = fs::read_dir(folder)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();

        let mut docs = GodotXmlDocs {
            class_fn_desc: HashMap::default(),
            regexes: Regexes::new(),
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
                // Parse members first, so more general docs for indexed accessors can be used
                // if they exist.
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
                        if let Some(property_name) = node.attribute("name") {
                            if !property_name.contains('/') {
                                if node.has_attribute("setter") {
                                    self.add_fn(class, &format!("set_{property_name}"), desc, &[]);
                                }
                                if node.has_attribute("getter") {
                                    self.add_fn(class, &format!("get_{property_name}"), desc, &[]);
                                }
                            }
                        }
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
            Self::reformat_as_rustdoc(&self.regexes, doc),
        );
    }

    // For types that godot-rust names differently than Godot
    fn translate_type(godot_type: &str) -> &str {
        // Note: there is some code duplication with Ty::from_src() in api.rs
        match godot_type {
            "String" => "GodotString",
            "Error" => "GodotError",
            "RID" => "Rid",
            "AABB" => "Aabb",
            "Array" => "VariantArray",
            "PoolByteArray" => "PoolArray<u8>",
            "PoolStringArray" => "PoolArray<GodotString>",
            "PoolVector2Array" => "PoolArray<Vector2>",
            "PoolVector3Array" => "PoolArray<Vector3>",
            "PoolColorArray" => "PoolArray<Color>",
            "PoolIntArray" => "PoolArray<i32>",
            "PoolRealArray" => "PoolArray<f32>",
            "G6DOFJointAxisParam" => "G6dofJointAxisParam",
            "G6DOFJointAxisFlag" => "G6dofJointAxisFlag",
            _ => godot_type,
        }
    }

    /// Takes the Godot documentation markup and transforms it to Rustdoc.
    /// Replaces BBCode syntax with Rustdoc/Markdown equivalents and implements working intra-doc links.
    fn reformat_as_rustdoc(re: &Regexes, godot_doc: String) -> String {
        // Note: there are still a few unsupported cases, such as:
        // * OK and ERR_CANT_CREATE (corresponding Result.Ok() and GodotError.ERR_CANT_CREATE)
        // * "indexed properties" which are not also exposed as getters, e.g. `gravity_point` in
        //   https://docs.godotengine.org/en/stable/classes/class_area2d.html#properties.
        //   This needs to be implemented first: https://github.com/godot-rust/godot-rust/issues/689

        // Info for GDScript blocks
        let godot_doc = if godot_doc.contains("[codeblock]") {
            format!("_Sample code is GDScript unless otherwise noted._\n\n{godot_doc}")
        } else {
            godot_doc
        };

        // Before any regex replacement, do verbatim replacements
        // Note: maybe some can be expressed as regex, but if text-replace does the job reliably enough, it's even faster
        let godot_doc = godot_doc
            .replace("[codeblock]", "```gdscript")
            .replace("[/codeblock]", "```")
            .replace("[code]", "`")
            .replace("[/code]", "`")
            .replace("[b]", "**")
            .replace("[/b]", "**")
            .replace("[i]", "_")
            .replace("[/i]", "_");

        // URLs
        let godot_doc = re.url.replace_all(&godot_doc, |c: &Captures| {
            let url = &c[1];
            let text = &c[2];

            if text.is_empty() {
                format!("<{url}>")
            } else {
                format!("[{text}]({url})")
            }
        });

        // [Type::member] style
        let godot_doc = re.class_member.replace_all(&godot_doc, |c: &Captures| {
            let godot_ty = &c[2];
            let rust_ty = Self::translate_type(godot_ty);

            format!(
                "[`{godot_ty}.{member}`][{rust_ty}::{member}]",
                godot_ty = godot_ty,
                rust_ty = rust_ty,
                member = &c[3]
            )
        });

        // [member] style
        let godot_doc = re.self_member.replace_all(&godot_doc, |c: &Captures| {
            format!("[`{member}`][Self::{member}]", member = &c[2])
        });

        // `member` style (no link)
        let godot_doc = re.no_link.replace_all(&godot_doc, |c: &Captures| {
            format!("`{member}`", member = &c[1])
        });

        // [Type] style
        let godot_doc = re.class.replace_all(&godot_doc, |c: &Captures| {
            let godot_ty = &c[2];
            let rust_ty = Self::translate_type(godot_ty);

            format!("[`{godot_ty}`][{rust_ty}]")
        });

        godot_doc.to_string()
    }
}

// Holds several compiled regexes to reuse across classes
// could also use 'lazy_regex' crate, but standard 'regex' has better IDE support and works well enough
struct Regexes {
    url: Regex,
    no_link: Regex,
    class: Regex,
    self_member: Regex,
    class_member: Regex,
}

impl Regexes {
    fn new() -> Self {
        Self {
            // Covers:
            // * [url=U]text[/url]
            // * [url=U][/url]
            url: Regex::new("\\[url=(.+?)](.*?)\\[/url]").unwrap(),

            // Covers:
            // * [code]C[/code]
            // * [signal C]
            // Must run before others, as [code] will itself match the link syntax
            no_link: Regex::new("\\[signal ([A-Za-z0-9_]+?)]").unwrap(),

            // Covers:
            // * [C]
            // * [enum C]
            class: Regex::new("\\[(enum )?([A-Za-z0-9_]+?)]").unwrap(),

            // Covers:
            // * [member M]
            // * [method M]
            // * [constant M]
            self_member: Regex::new("\\[(member|method|constant) ([A-Za-z0-9_]+?)]").unwrap(),

            // Covers:
            // * [member C.M]
            // * [method C.M]
            // * [constant C.M]
            class_member: Regex::new(
                "\\[(member|method|constant) ([A-Za-z0-9_]+?)\\.([A-Za-z0-9_]+?)]",
            )
            .unwrap(),
        }
    }
}

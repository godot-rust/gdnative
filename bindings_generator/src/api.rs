use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};

use miniserde::Deserialize;
miniserde::make_place!(Place);

pub struct Api {
    pub classes: Vec<GodotClass>,
    pub api_underscore: HashSet<String>,
}

impl Api {
    pub fn new() -> Self {
        let mut api = Api {
            classes: miniserde::json::from_str(get_api_json())
                .expect("Failed to parse the API description"),
            api_underscore: Default::default(),
        };

        api.strip_leading_underscores();

        api
    }

    pub fn find_class<'a, 'b>(&'a self, name: &'b str) -> Option<&'a GodotClass> {
        for class in &self.classes {
            if class.name == name {
                return Some(class);
            }
        }

        None
    }

    pub fn class_inherits(&self, class: &GodotClass, base_class_name: &str) -> bool {
        if class.base_class == base_class_name {
            return true;
        }

        if let Some(parent) = self.find_class(&class.base_class) {
            return self.class_inherits(parent, base_class_name);
        }

        false
    }

    fn strip_leading_underscores(&mut self) {
        for class in &mut self.classes {
            if class.name.starts_with('_') {
                class.name = class.name[1..].to_string();
                self.api_underscore.insert(class.name.clone());
            }
            for method in &mut class.methods {
                if method.return_type.starts_with('_') {
                    method.return_type = method.return_type[1..].to_string();
                }
                for arg in &mut method.arguments {
                    if arg.ty.starts_with('_') {
                        arg.ty = arg.ty[1..].to_string();
                    }
                }
            }
        }
    }
}

impl Default for Api {
    fn default() -> Self {
        Api::new()
    }
}

#[derive(Deserialize, Debug)]
pub struct GodotClass {
    pub name: String,
    pub base_class: String,
    pub api_type: String,
    pub singleton: bool,
    pub is_reference: bool,
    #[serde(rename = "instanciable")]
    pub instantiable: bool,

    pub properties: Vec<Property>,
    pub methods: Vec<GodotMethod>,
    pub enums: Vec<Enum>,
    pub constants: HashMap<ConstantName, ConstantValue>,
}

impl GodotClass {
    pub fn is_refcounted(&self) -> bool {
        self.is_reference || &self.name == "Reference"
    }

    pub fn is_pointer_safe(&self) -> bool {
        self.is_refcounted() || self.singleton
    }

    pub fn is_getter(&self, name: &str) -> bool {
        self.properties.iter().any(|p| p.getter == name)
    }
}

pub type ConstantName = String;
pub type ConstantValue = i64;

#[derive(PartialEq, Eq, Deserialize, Debug)]
pub struct Enum {
    pub name: String,
    pub values: HashMap<String, i64>,
}

impl core::cmp::Ord for Enum {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        core::cmp::Ord::cmp(&self.name, &other.name)
    }
}

impl core::cmp::PartialOrd for Enum {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        core::cmp::PartialOrd::partial_cmp(&self.name, &other.name)
    }
}

#[derive(Deserialize, Debug)]
pub struct Property {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub getter: String,
    pub setter: String,
    pub index: i64,
}

#[derive(Deserialize, Debug)]
pub struct GodotMethod {
    pub name: String,
    pub return_type: String,

    pub is_editor: bool,
    pub is_noscript: bool,
    pub is_const: bool,
    pub is_reverse: bool,
    pub is_virtual: bool,
    pub has_varargs: bool,

    pub arguments: Vec<GodotArgument>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MethodName<'a> {
    pub rust_name: &'a str,
    pub original_name: &'a str,
}

impl GodotMethod {
    pub fn get_name(&self) -> MethodName {
        // GDScript and NativeScript have ::new methods but we want to reserve
        // the name for the constructors.
        if &self.name == "new" {
            return MethodName {
                rust_name: "_new",
                original_name: "new",
            };
        }

        MethodName {
            rust_name: &self.name,
            original_name: &self.name,
        }
    }

    pub fn get_return_type(&self) -> Ty {
        Ty::from_src(&self.return_type)
    }
}

#[derive(Deserialize, Debug)]
pub struct GodotArgument {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub has_default_value: bool,
    pub default_value: String,
}

impl GodotArgument {
    pub fn get_type(&self) -> Ty {
        Ty::from_src(&self.ty)
    }
}

#[derive(Clone)]
pub enum Ty {
    Void,
    String,
    F64,
    I64,
    Bool,
    Vector2,
    Vector3,
    Quat,
    Transform,
    Transform2D,
    Rect2,
    Plane,
    Basis,
    Color,
    NodePath,
    Variant,
    Aabb,
    Rid,
    VariantArray,
    Dictionary,
    ByteArray,
    StringArray,
    Vector2Array,
    Vector3Array,
    ColorArray,
    Int32Array,
    Float32Array,
    Result,
    VariantType,
    VariantOperator,
    Enum(String),
    Object(String),
}

impl Ty {
    pub fn from_src(src: &str) -> Self {
        match src {
            "void" => Ty::Void,
            "String" => Ty::String,
            "float" => Ty::F64,
            "int" => Ty::I64,
            "bool" => Ty::Bool,
            "Vector2" => Ty::Vector2,
            "Vector3" => Ty::Vector3,
            "Quat" => Ty::Quat,
            "Transform" => Ty::Transform,
            "Transform2D" => Ty::Transform2D,
            "Rect2" => Ty::Rect2,
            "Plane" => Ty::Plane,
            "Basis" => Ty::Basis,
            "Color" => Ty::Color,
            "NodePath" => Ty::NodePath,
            "Variant" => Ty::Variant,
            "AABB" => Ty::Aabb,
            "RID" => Ty::Rid,
            "Array" => Ty::VariantArray,
            "Dictionary" => Ty::Dictionary,
            "PoolByteArray" => Ty::ByteArray,
            "PoolStringArray" => Ty::StringArray,
            "PoolVector2Array" => Ty::Vector2Array,
            "PoolVector3Array" => Ty::Vector3Array,
            "PoolColorArray" => Ty::ColorArray,
            "PoolIntArray" => Ty::Int32Array,
            "PoolRealArray" => Ty::Float32Array,
            "enum.Error" => Ty::Result,
            "enum.Variant::Type" => Ty::VariantType,
            "enum.Variant::Operator" => Ty::VariantOperator,
            ty if ty.starts_with("enum.") => {
                let mut split = ty[5..].split("::");
                let mut class = split.next().unwrap();
                if class.starts_with('_') {
                    class = &class[1..];
                }
                let name = split.next().unwrap();
                Ty::Enum(format!("{}{}", class, name))
            }
            ty => Ty::Object(ty.into()),
        }
    }

    pub fn to_rust(&self) -> syn::Type {
        match self {
            Ty::Void => syn::parse_quote! {()},
            Ty::String => syn::parse_quote! { GodotString },
            Ty::F64 => syn::parse_quote! { f64 },
            Ty::I64 => syn::parse_quote! { i64 },
            Ty::Bool => syn::parse_quote! { bool },
            Ty::Vector2 => syn::parse_quote! { Vector2 },
            Ty::Vector3 => syn::parse_quote! { Vector3 },
            Ty::Quat => syn::parse_quote! { Quat },
            Ty::Transform => syn::parse_quote! { Transform },
            Ty::Transform2D => syn::parse_quote! { Transform2D },
            Ty::Rect2 => syn::parse_quote! { Rect2 },
            Ty::Plane => syn::parse_quote! { Plane },
            Ty::Basis => syn::parse_quote! { Basis },
            Ty::Color => syn::parse_quote! { Color },
            Ty::NodePath => syn::parse_quote! { NodePath },
            Ty::Variant => syn::parse_quote! { Variant },
            Ty::Aabb => syn::parse_quote! { Aabb },
            Ty::Rid => syn::parse_quote! { Rid },
            Ty::VariantArray => syn::parse_quote! { VariantArray },
            Ty::Dictionary => syn::parse_quote! { Dictionary },
            Ty::ByteArray => syn::parse_quote! { ByteArray },
            Ty::StringArray => syn::parse_quote! { StringArray },
            Ty::Vector2Array => syn::parse_quote! { Vector2Array },
            Ty::Vector3Array => syn::parse_quote! { Vector3Array },
            Ty::ColorArray => syn::parse_quote! { ColorArray },
            Ty::Int32Array => syn::parse_quote! { Int32Array },
            Ty::Float32Array => syn::parse_quote! { Float32Array },
            Ty::Result => syn::parse_quote! { GodotResult },
            Ty::VariantType => syn::parse_quote! { VariantType },
            Ty::VariantOperator => syn::parse_quote! { VariantOperator },
            Ty::Enum(ref name) => {
                let name = format_ident!("{}", name);
                syn::parse_quote! { #name }
            }
            Ty::Object(ref name) => {
                let name = format_ident!("{}", name);
                syn::parse_quote! { Option<Ref<#name, thread_access::Shared>> }
            }
        }
    }

    pub fn to_rust_arg(&self) -> syn::Type {
        match self {
            Ty::Object(ref name) => {
                let name = format_ident!("{}", name);
                syn::parse_quote! { impl AsArg<Target = #name> }
            }
            _ => self.to_rust(),
        }
    }

    pub fn to_sys(&self) -> Option<syn::Type> {
        match self {
            Ty::Void => None,
            Ty::String => Some(syn::parse_quote! { sys::godot_string }),
            Ty::F64 => Some(syn::parse_quote! { sys::godot_real }),
            Ty::I64 => Some(syn::parse_quote! { sys::godot_int }),
            Ty::Bool => Some(syn::parse_quote! { sys::godot_bool }),
            Ty::Vector2 => Some(syn::parse_quote! { sys::godot_vector2 }),
            Ty::Vector3 => Some(syn::parse_quote! { sys::godot_vector3 }),
            Ty::Quat => Some(syn::parse_quote! { sys::godot_quat }),
            Ty::Transform => Some(syn::parse_quote! { sys::godot_transform }),
            Ty::Transform2D => Some(syn::parse_quote! { sys::godot_transform2d }),
            Ty::Rect2 => Some(syn::parse_quote! { sys::godot_rect2 }),
            Ty::Plane => Some(syn::parse_quote! { sys::godot_plane }),
            Ty::Basis => Some(syn::parse_quote! { sys::godot_basis }),
            Ty::Color => Some(syn::parse_quote! { sys::godot_color }),
            Ty::NodePath => Some(syn::parse_quote! { sys::godot_node_path }),
            Ty::Variant => Some(syn::parse_quote! { sys::godot_variant }),
            Ty::Aabb => Some(syn::parse_quote! { sys::godot_aabb }),
            Ty::Rid => Some(syn::parse_quote! { sys::godot_rid }),
            Ty::VariantArray => Some(syn::parse_quote! { sys::godot_array }),
            Ty::Dictionary => Some(syn::parse_quote! { sys::godot_dictionary }),
            Ty::ByteArray => Some(syn::parse_quote! { sys::godot_pool_byte_array }),
            Ty::StringArray => Some(syn::parse_quote! { sys::godot_pool_string_array }),
            Ty::Vector2Array => Some(syn::parse_quote! { sys::godot_pool_vector2_array }),
            Ty::Vector3Array => Some(syn::parse_quote! { sys::godot_pool_vector3_array }),
            Ty::ColorArray => Some(syn::parse_quote! { sys::godot_pool_color_array }),
            Ty::Int32Array => Some(syn::parse_quote! { sys::godot_pool_int_array }),
            Ty::Float32Array => Some(syn::parse_quote! { sys::godot_pool_real_array }),
            Ty::Result => Some(syn::parse_quote! { sys::godot_error }),
            Ty::VariantType => Some(syn::parse_quote! { sys::variant_type }),
            Ty::VariantOperator => Some(syn::parse_quote! { sys::godot_variant_operator }),
            Ty::Enum(_) => None,
            Ty::Object(_) => Some(syn::parse_quote! { sys::godot_object }),
        }
    }

    pub fn to_return_post(&self) -> TokenStream {
        match self {
            Ty::Void => Default::default(),
            Ty::F64 | &Ty::I64 | &Ty::Bool | &Ty::Enum(_) => {
                quote! { ret }
            }
            Ty::Vector2
            | Ty::Vector3
            | Ty::Transform
            | Ty::Transform2D
            | Ty::Quat
            | Ty::Aabb
            | Ty::Rect2
            | Ty::Basis
            | Ty::Plane
            | Ty::Color => {
                quote! { mem::transmute(ret) }
            }
            Ty::Rid => {
                quote! { Rid::from_sys(ret) }
            }
            Ty::String
            | Ty::NodePath
            | Ty::VariantArray
            | Ty::Dictionary
            | Ty::ByteArray
            | Ty::StringArray
            | Ty::Vector2Array
            | Ty::Vector3Array
            | Ty::ColorArray
            | Ty::Int32Array
            | Ty::Float32Array
            | Ty::Variant => {
                let rust_ty = self.to_rust();
                quote! {
                    #rust_ty::from_sys(ret)
                }
            }
            Ty::Object(ref name) => {
                let name = format_ident!("{}", name);
                quote! {
                    ptr::NonNull::new(ret)
                        .map(|sys| <Ref<#name, thread_access::Shared>>::move_from_sys(sys))
                }
            }
            Ty::Result => {
                quote! { GodotError::result_from_sys(ret) }
            }
            Ty::VariantType => {
                quote! { VariantType::from_sys(ret) }
            }
            Ty::VariantOperator => {
                quote! {
                    VariantOperator::try_from_sys(ret).expect("enum variant should be valid")
                }
            }
        }
    }
}

pub fn get_api_json() -> &'static str {
    include_str!("../api.json")
}

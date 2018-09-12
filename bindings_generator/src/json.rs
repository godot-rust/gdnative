use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct GodotClass {
    pub name: String,
    pub base_class: String,
    pub api_type: String,
    pub singleton: bool,
    pub is_reference: bool,
    pub instanciable: bool,

    pub methods: Vec<GodotMethod>,
    pub enums: Vec<Enum>,
}

impl GodotClass {
    pub fn is_refcounted(&self) -> bool { self.is_reference || &self.name == "Reference" }

    pub fn is_pointer_safe(&self) -> bool { self.is_refcounted() || self.singleton }
}

#[derive(Deserialize, Debug)]
pub struct Enum {
    pub name: String,
    pub values: HashMap<String, u32>,
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

impl GodotMethod {
    pub fn get_name(&self) -> &str {
        // GDScript and NativeScript have ::new methods but we want to reserve
        // the name for the constructors.
        if &self.name == "new" {
            return "_new";
        }

        &self.name
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
            ty if ty.starts_with("enum.") => {
                let mut split = ty[5..].split("::");
                let class = split.next().unwrap();
                let name = split.next().unwrap();
                Ty::Enum(format!("{}{}", class, name))
            }
            ty => {
                Ty::Object(ty.into())
            },
        }
    }

    pub fn to_rust(&self) -> Option<String> {
        match self {
            &Ty::Void => Some(String::from("()")),
            &Ty::String => Some(String::from("GodotString")),
            &Ty::F64 => Some(String::from("f64")),
            &Ty::I64 => Some(String::from("i64")),
            &Ty::Bool => Some(String::from("bool")),
            &Ty::Vector2 => Some(String::from("Vector2")),
            &Ty::Vector3 => Some(String::from("Vector3")),
            &Ty::Quat => Some(String::from("Quat")),
            &Ty::Transform => Some(String::from("Transform")),
            &Ty::Transform2D => Some(String::from("Transform2D")),
            &Ty::Rect2 => Some(String::from("Rect2")),
            &Ty::Plane => Some(String::from("Plane")),
            &Ty::Basis => Some(String::from("Basis")),
            &Ty::Color => Some(String::from("Color")),
            &Ty::NodePath => Some(String::from("NodePath")),
            &Ty::Variant => Some(String::from("Variant")),
            &Ty::Aabb => Some(String::from("Aabb")),
            &Ty::Rid => Some(String::from("Rid")),
            &Ty::VariantArray => Some(String::from("VariantArray")),
            &Ty::Dictionary => Some(String::from("Dictionary")),
            &Ty::ByteArray => Some(String::from("ByteArray")),
            &Ty::StringArray => Some(String::from("StringArray")),
            &Ty::Vector2Array => Some(String::from("Vector2Array")),
            &Ty::Vector3Array => Some(String::from("Vector3Array")),
            &Ty::ColorArray => Some(String::from("ColorArray")),
            &Ty::Int32Array => Some(String::from("Int32Array")),
            &Ty::Float32Array => Some(String::from("Float32Array")),
            &Ty::Result => Some(String::from("GodotResult")),
            &Ty::VariantType => Some(String::from("VariantType")),
            &Ty::Enum(ref name) => Some(String::from(name.clone())),
            &Ty::Object(ref name) => Some(format!("Option<{}>", name)),
        }
    }

    pub fn to_sys(&self) -> Option<String> {
        match self {
            &Ty::Void => None,
            &Ty::String => Some(String::from("sys::godot_string")),
            &Ty::F64 => Some(String::from("sys::godot_real")),
            &Ty::I64 => Some(String::from("sys::godot_int")),
            &Ty::Bool => Some(String::from("sys::godot_bool")),
            &Ty::Vector2 => Some(String::from("sys::godot_vector2")),
            &Ty::Vector3 => Some(String::from("sys::godot_vector3")),
            &Ty::Quat => Some(String::from("sys::godot_quat")),
            &Ty::Transform => Some(String::from("sys::godot_transform")),
            &Ty::Transform2D => Some(String::from("sys::godot_transform2d")),
            &Ty::Rect2 => Some(String::from("sys::godot_rect2")),
            &Ty::Plane => Some(String::from("sys::godot_plane")),
            &Ty::Basis => Some(String::from("sys::godot_basis")),
            &Ty::Color => Some(String::from("sys::godot_color")),
            &Ty::NodePath => Some(String::from("sys::godot_node_path")),
            &Ty::Variant => Some(String::from("sys::godot_variant")),
            &Ty::Aabb => Some(String::from("sys::godot_aabb")),
            &Ty::Rid => Some(String::from("sys::godot_rid")),
            &Ty::VariantArray => Some(String::from("sys::godot_array")),
            &Ty::Dictionary => Some(String::from("sys::godot_dictionary")),
            &Ty::ByteArray => Some(String::from("sys::godot_pool_byte_array")),
            &Ty::StringArray => Some(String::from("sys::godot_pool_string_array")),
            &Ty::Vector2Array => Some(String::from("sys::godot_pool_vector2_array")),
            &Ty::Vector3Array => Some(String::from("sys::godot_pool_vector3_array")),
            &Ty::ColorArray => Some(String::from("sys::godot_pool_color_array")),
            &Ty::Int32Array => Some(String::from("sys::godot_pool_int_array")),
            &Ty::Float32Array => Some(String::from("sys::godot_pool_real_array")),
            &Ty::Result => Some(String::from("sys::godot_error")),
            &Ty::VariantType => Some(String::from("sys::variant_type")),
            &Ty::Enum(_) => None, // TODO
            &Ty::Object(_) => Some(String::from("sys::godot_object")),
        }
    }
}

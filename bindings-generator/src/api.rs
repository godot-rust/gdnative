use super::classes::generate_enum_name;
use miniserde::Deserialize;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};

miniserde::make_place!(Place);

pub struct Api {
    pub classes: Vec<GodotClass>,
    pub api_underscore: HashSet<String>,
}

impl Api {
    /// Construct an `Api` instance from JSON data.
    ///
    /// The JSON data can be generated from Godot using the following command:
    ///
    /// `/path/to/godot --gdnative-generate-json-api /path/to/api.json`
    ///
    /// # Panics
    ///
    /// If the `data` is not valid JSON data the function will panic.
    pub fn new(data: &str) -> Self {
        let mut api = Self {
            classes: miniserde::json::from_str(data).expect("Invalid JSON data"),
            api_underscore: Default::default(),
        };

        api.strip_leading_underscores();
        api.generate_module_names();

        api.classes
            .iter_mut()
            .flat_map(|class| class.enums.iter_mut())
            .for_each(|e| e.strip_common_prefix());

        api
    }

    pub fn find_class(&self, name: &str) -> Option<&GodotClass> {
        self.classes.iter().find(|&class| class.name == name)
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

    fn generate_module_names(&mut self) {
        self.classes
            .iter_mut()
            .for_each(|class| class.generate_module_name());
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

    module_name: Option<String>,
    base_class_module_name: Option<String>,
}

impl GodotClass {
    pub fn generate_module_name(&mut self) {
        let module_name = module_name_from_class_name(&self.name);
        self.module_name.replace(module_name);

        let base_class_module_name = module_name_from_class_name(&self.base_class);
        self.base_class_module_name.replace(base_class_module_name);
    }

    pub fn module(&self) -> &str {
        self.module_name
            .as_ref()
            .expect("Module Names should have been generated.")
    }

    pub fn base_class_module(&self) -> &str {
        self.base_class_module_name
            .as_ref()
            .expect("Module Names should have been generated.")
    }

    /// Returns the name of the base class if `base_class` is not empty. Returns `None` otherwise.
    pub fn base_class_name(&self) -> Option<&str> {
        if self.base_class.is_empty() {
            None
        } else {
            Some(&self.base_class)
        }
    }

    pub fn is_singleton_thread_safe(&self) -> bool {
        assert!(self.singleton, "class is not a singleton");
        !matches!(
            self.name.as_str(),
            "VisualServer" | "PhysicsServer" | "Physics3DServer" | "Physics2DServer"
        )
    }

    /// Returns the base class from `api` if `base_class` is not empty. Returns `None` otherwise.
    pub fn base_class<'a>(&self, api: &'a Api) -> Option<&'a Self> {
        self.base_class_name()
            .map(|name| api.find_class(name).expect("base class should exist"))
    }

    pub fn is_refcounted(&self) -> bool {
        self.is_reference || &self.name == "Reference"
    }

    pub fn is_pointer_safe(&self) -> bool {
        self.is_refcounted() || self.singleton
    }

    pub fn is_getter(&self, name: &str) -> bool {
        self.properties.iter().any(|p| p.getter == name)
    }

    /// Whether there is a snake_case module containing related symbols (nested types in C++)
    pub fn has_related_module(&self) -> bool {
        !self.enums.is_empty()
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

impl Enum {
    pub fn strip_common_prefix(&mut self) {
        // If there is only 1 variant, there are no 'common' prefixes.
        if self.values.len() <= 1 {
            return;
        }

        let mut variants: Vec<_> = self.values.iter().map(|v| (&v.0[..], *v.1)).collect();

        // Build a map of prefix to occurrence
        loop {
            let underscore_index = variants[0].0.chars().enumerate().find_map(|(index, ch)| {
                if ch == '_' {
                    Some(index)
                } else {
                    None
                }
            });

            let underscore_index = match underscore_index {
                Some(index) if index > 0 => index,
                Some(_) | None => break,
            };

            // Get a slice up to and including the `_`
            let prefix = &variants[0].0[..=underscore_index];

            if !variants.iter().all(|v| v.0.starts_with(prefix)) {
                break;
            }

            // remove common prefix from variants
            variants.iter_mut().for_each(|(ref mut name, _)| {
                *name = &name[prefix.len()..];
            });
        }

        let new_variants: HashMap<_, _> = variants
            .into_iter()
            .map(|(name, value)| {
                let starts_with_number = name
                    .chars()
                    .next()
                    .expect("name should not be empty")
                    .is_numeric();

                let capacity = if starts_with_number {
                    name.len() + 1
                } else {
                    name.len()
                };

                let mut n = String::with_capacity(capacity);

                // prefix numeric enum variants with `_`
                if starts_with_number {
                    n.push('_');
                }

                n.push_str(name);
                (n, value)
            })
            .collect();

        self.values = new_variants;
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

#[derive(Clone, Eq, PartialEq)]
pub enum Ty {
    Void,
    String,
    F64,
    I64,
    Bool,
    Vector2,
    Vector3,
    Vector3Axis,
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
    Enum(syn::TypePath),
    Object(syn::TypePath),
}

impl Ty {
    // Note: there is some code duplication with GodotXmlDocs::translate_type() in class_docs.rs
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
            "enum.Vector3::Axis" => Ty::Vector3Axis,
            ty if ty.starts_with("enum.") => {
                // Enums may reference known types (above list), check if it's a known type first
                let mut split = ty[5..].split("::");
                let class_name = split.next().unwrap();
                let enum_raw_name = split.next().unwrap();
                let name = format_ident!("{}", generate_enum_name(class_name, enum_raw_name));
                let module = format_ident!("{}", module_name_from_class_name(class_name));
                // Is it a known type?
                match Ty::from_src(class_name) {
                    Ty::Enum(_) | Ty::Object(_) => {
                        Ty::Enum(syn::parse_quote! { crate::generated::#module::#name })
                    }
                    _ => Ty::Enum(syn::parse_quote! { #module::#name }),
                }
            }
            ty => {
                let ty = format_ident!("{}", ty);
                Ty::Object(syn::parse_quote! { crate::generated::#ty })
            }
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
            Ty::Vector3Axis => syn::parse_quote! { Axis },
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
            Ty::ByteArray => syn::parse_quote! { PoolArray<u8> },
            Ty::StringArray => syn::parse_quote! { PoolArray<GodotString> },
            Ty::Vector2Array => syn::parse_quote! { PoolArray<Vector2> },
            Ty::Vector3Array => syn::parse_quote! { PoolArray<Vector3> },
            Ty::ColorArray => syn::parse_quote! { PoolArray<Color> },
            Ty::Int32Array => syn::parse_quote! { PoolArray<i32> },
            Ty::Float32Array => syn::parse_quote! { PoolArray<f32> },
            Ty::Result => syn::parse_quote! { GodotResult },
            Ty::VariantType => syn::parse_quote! { VariantType },
            Ty::VariantOperator => syn::parse_quote! { VariantOperator },
            Ty::Enum(path) => syn::parse_quote! { #path },
            Ty::Object(path) => {
                syn::parse_quote! { Option<Ref<#path, ownership::Shared>> }
            }
        }
    }

    pub fn to_rust_arg(&self) -> syn::Type {
        match self {
            Ty::Variant => syn::parse_quote! { impl OwnedToVariant },
            Ty::NodePath => syn::parse_quote! { impl Into<NodePath> },
            Ty::String => syn::parse_quote! { impl Into<GodotString> },
            Ty::Object(ref name) => {
                syn::parse_quote! { impl AsArg<#name> }
            }
            _ => self.to_rust(),
        }
    }

    pub fn to_icall_arg(&self) -> syn::Type {
        match self {
            Ty::Object(_) => syn::parse_quote! { *mut sys::godot_object },
            _ => self.to_rust(),
        }
    }

    pub fn to_icall_return(&self) -> syn::Type {
        match self {
            Ty::Void => syn::parse_quote! { () },
            Ty::String => syn::parse_quote! { sys::godot_string },
            Ty::F64 => syn::parse_quote! { f64 },
            Ty::I64 => syn::parse_quote! { i64 },
            Ty::Bool => syn::parse_quote! { sys::godot_bool },
            Ty::Vector2 => syn::parse_quote! { sys::godot_vector2 },
            Ty::Vector3 => syn::parse_quote! { sys::godot_vector3 },

            Ty::Quat => syn::parse_quote! { sys::godot_quat },
            Ty::Transform => syn::parse_quote! { sys::godot_transform },
            Ty::Transform2D => syn::parse_quote! { sys::godot_transform2d },
            Ty::Rect2 => syn::parse_quote! { sys::godot_rect2 },
            Ty::Plane => syn::parse_quote! { sys::godot_plane },
            Ty::Basis => syn::parse_quote! { sys::godot_basis },
            Ty::Color => syn::parse_quote! { sys::godot_color },
            Ty::NodePath => syn::parse_quote! { sys::godot_node_path },
            Ty::Variant => syn::parse_quote! { sys::godot_variant },
            Ty::Aabb => syn::parse_quote! { sys::godot_aabb },
            Ty::Rid => syn::parse_quote! { sys::godot_rid },
            Ty::VariantArray => syn::parse_quote! { sys::godot_array },
            Ty::Dictionary => syn::parse_quote! { sys::godot_dictionary },
            Ty::ByteArray => syn::parse_quote! { sys::godot_pool_byte_array },
            Ty::StringArray => syn::parse_quote! { sys::godot_pool_string_array },
            Ty::Vector2Array => syn::parse_quote! { sys::godot_pool_vector2_array },
            Ty::Vector3Array => syn::parse_quote! { sys::godot_pool_vector3_array },
            Ty::ColorArray => syn::parse_quote! { sys::godot_pool_color_array },
            Ty::Int32Array => syn::parse_quote! { sys::godot_pool_int_array },
            Ty::Float32Array => syn::parse_quote! { sys::godot_pool_real_array },

            Ty::Vector3Axis | Ty::Result | Ty::VariantType | Ty::VariantOperator | Ty::Enum(_) => {
                syn::parse_quote! { i64 }
            }

            Ty::Object(_) => syn::parse_quote! { *mut sys::godot_object },
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
            Ty::Vector3Axis => None,
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
            Ty::F64 | Ty::I64 | Ty::Bool => {
                quote! { ret as _ }
            }
            Ty::Enum(path) => {
                quote! { #path(ret) }
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
            Ty::Vector3Axis => {
                quote! { mem::transmute(ret as u32) }
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
                    <#rust_ty>::from_sys(ret)
                }
            }
            Ty::Object(ref path) => {
                quote! {
                    ptr::NonNull::new(ret)
                        .map(|sys| <Ref<#path, ownership::Shared>>::move_from_sys(sys))
                }
            }
            Ty::Result => {
                quote! { GodotError::result_from_sys(ret as _) }
            }
            Ty::VariantType => {
                quote! { VariantType::from_sys(ret as _) }
            }
            Ty::VariantOperator => {
                quote! {
                    VariantOperator::try_from_sys(ret as _).expect("enum variant should be valid")
                }
            }
        }
    }

    pub fn to_return_post_variant(&self) -> TokenStream {
        match self {
            Ty::Void => Default::default(),
            Ty::F64 | Ty::I64 | Ty::Bool => {
                let rust_type = self.to_rust();
                quote! {
                    <#rust_type>::coerce_from_variant(&ret)
                }
            }

            // The `sys` type aliases here can point to different types depending on the platform.
            // Do not simplify to (Linux) primitive types.
            Ty::Result => {
                quote! { GodotError::result_from_sys(sys::godot_error::from_variant(&ret).expect("Unexpected variant type")) }
            }
            Ty::VariantType => {
                quote! { VariantType::from_sys(sys::godot_variant_type::from_variant(&ret).expect("Unexpected variant type")) }
            }
            Ty::VariantOperator => {
                quote! {
                    VariantOperator::try_from_sys(sys::godot_variant_operator::from_variant(&ret).expect("Unexpected variant type"))
                        .expect("enum variant should be valid")
                }
            }

            Ty::Vector3Axis => {
                quote! {
                    unsafe {
                        mem::transmute::<u32, Axis>(u32::from_variant(&ret).expect("Unexpected variant type") as _)
                    }
                }
            }
            _ => {
                let rust_type = self.to_rust();
                // always a variant returned, use FromVariant
                quote! {
                    <#rust_type>::from_variant(&ret).expect("Unexpected variant type")
                }
            }
        }
    }
}

pub fn module_name_from_class_name(class_name: &str) -> String {
    // Remove underscores and make peekable
    let mut class_chars = class_name.bytes().filter(|&ch| ch != b'_').peekable();

    // 2-lookbehind
    let mut previous: [Option<u8>; 2] = [None, None]; // previous-previous, previous

    // None is not upper or number
    #[inline(always)]
    fn up_or_num<T>(ch: T) -> bool
    where
        T: Into<Option<u8>>,
    {
        let ch = ch.into();
        match ch {
            Some(ch) => ch.is_ascii_digit() || ch.is_ascii_uppercase(),
            None => false,
        }
    }

    // None is lowercase
    #[inline(always)]
    fn is_lowercase_or<'a, T>(ch: T, default: bool) -> bool
    where
        T: Into<Option<&'a u8>>,
    {
        let ch = ch.into();
        match ch {
            Some(ch) => ch.is_ascii_lowercase(),
            None => default,
        }
    }

    let mut result = Vec::with_capacity(class_name.len());
    while let Some(current) = class_chars.next() {
        let next = class_chars.peek();

        let [two_prev, one_prev] = previous;

        // See tests for cases covered
        let caps_to_lowercase = up_or_num(one_prev)
            && up_or_num(current)
            && is_lowercase_or(next, false)
            && !is_lowercase_or(&two_prev, true);

        // Add an underscore for Lowercase folowed by Uppercase|Num
        // Node2D => node_2d (numbers are considered uppercase)
        let lower_to_uppercase = is_lowercase_or(&one_prev, false) && up_or_num(current);

        if caps_to_lowercase || lower_to_uppercase {
            result.push(b'_');
        }
        result.push(current.to_ascii_lowercase());

        // Update the look-behind
        previous = [previous[1], Some(current)];
    }

    let mut result = String::from_utf8(result).unwrap();

    // There are a few cases where the conversions do not work:
    // * VisualShaderNodeVec3Uniform => visual_shader_node_vec_3_uniform
    // * VisualShaderNodeVec3Constant => visual_shader_node_vec_3_constant
    if let Some(range) = result.find("_vec_3").map(|i| i..i + 6) {
        result.replace_range(range, "_vec3_")
    }
    if let Some(range) = result.find("gd_native").map(|i| i..i + 9) {
        result.replace_range(range, "gdnative")
    }
    if let Some(range) = result.find("gd_script").map(|i| i..i + 9) {
        result.replace_range(range, "gdscript")
    }

    // To prevent clobbering `gdnative` during a glob import we rename it to `gdnative_`
    if result == "gdnative" {
        return "gdnative_".into();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_name_generator() {
        let tests = vec![
            // A number of test cases to cover some possibilities:
            // * Underscores are removed
            // * First character is always lowercased
            // * lowercase to an uppercase inserts an underscore
            //   - FooBar => foo_bar
            // * two capital letter words does not separate the capital letters:
            //   - FooBBaz => foo_bbaz (lower, cap, cap, lower)
            // * many-capital letters to lowercase inserts an underscore before the last uppercase letter:
            //   - FOOBar => boo_bar
            // underscores
            ("Ab_Cdefg", "ab_cdefg"),
            ("_Abcd", "abcd"),
            ("Abcd_", "abcd"),
            // first and last
            ("Abcdefg", "abcdefg"),
            ("abcdefG", "abcdef_g"),
            // more than 2 caps
            ("ABCDefg", "abc_defg"),
            ("AbcDEFg", "abc_de_fg"),
            ("AbcdEF10", "abcd_ef10"),
            ("AbcDEFG", "abc_defg"),
            ("ABCDEFG", "abcdefg"),
            ("ABC", "abc"),
            // Lowercase to an uppercase
            ("AbcDefg", "abc_defg"),
            // Only 2 caps
            ("ABcdefg", "abcdefg"),
            ("ABcde2G", "abcde_2g"),
            ("AbcDEfg", "abc_defg"),
            ("ABcDe2G", "abc_de_2g"),
            ("abcdeFG", "abcde_fg"),
            ("AB", "ab"),
            // Lowercase to an uppercase
            ("AbcdefG", "abcdef_g"), // PosX => pos_x
            // text changes
            ("FooVec3Uni", "foo_vec3_uni"),
            ("GDNative", "gdnative_"),
            ("GDScript", "gdscript"),
        ];
        tests.iter().for_each(|(class_name, expected)| {
            let actual = module_name_from_class_name(class_name);
            assert_eq!(*expected, actual, "Input: {class_name}");
        });
    }
}

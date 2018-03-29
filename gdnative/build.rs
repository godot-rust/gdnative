
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate heck;

use heck::CamelCase;

use std::fs::File;
use std::env;
use std::path::PathBuf;
use std::io::Write;
use std::fmt;

use std::collections::HashMap;

#[derive(Clone)]
enum Ty {
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
            ty if ty.starts_with("enum.") => Ty::Enum(ty[5..].into()),
            ty => {
                Ty::Object(ty.into())
            },
        }
    }

    fn to_rust(&self) -> Option<String> {
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
            &Ty::Enum(_) => None, // TODO
            &Ty::Object(ref name) => Some(format!("Option<GodotRef<{}>>", name)),
        }
    }

    fn to_sys(&self) -> Option<String> {
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

fn main() {
    let classes: Vec<GodotClass> = serde_json::from_reader(File::open("api.json").unwrap())
        .expect("Failed to parse api.json");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut output = File::create(out_path.join("types.rs")).unwrap();

    writeln!(output, "use std::ptr;").unwrap();

    for class in classes {
        writeln!(output, r#"
#[allow(non_camel_case_types)]
pub struct {name} {{
    info: GodotClassInfo,
"#, name = class.name).unwrap();
        if class.base_class != "" {
            writeln!(output, r#"
    parent: {parent},
            "#, parent = class.base_class).unwrap();
        }
        writeln!(output, r#"
}}

"#).unwrap();

        for e in &class.enums {
            // TODO: check whether the start of the variant name is
            // equal to the end of the enum name and if so don't repeat it
            // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.
            writeln!(output, r#"
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum {class_name}{enum_name} {{
"#, class_name = class.name, enum_name = e.name
            ).unwrap();

            for (key, val) in &e.values {
                let key = key.as_str().to_camel_case();
                writeln!(output,
r#"    {key} = {val},"#,
                key = key.as_str().to_camel_case(), val = val).unwrap();
            }
            writeln!(output, r#"
}}"#
            ).unwrap();
        }

        writeln!(output, r#"

unsafe impl GodotClass for {name} {{
    type ClassData = {name};
    type Reference = {name};

    fn godot_name() -> &'static str {{
        "{name}"
    }}

    unsafe fn register_class(_desc: *mut libc::c_void) {{
        panic!("Can't register");
    }}

    fn godot_info(&self) -> &GodotClassInfo {{
        &self.info
    }}
    unsafe fn reference(_this: *mut sys::godot_object, data: &Self::ClassData) -> &Self::Reference {{
        data
    }}
    unsafe fn from_object(obj: *mut sys::godot_object) -> Self {{
        {name} {{
            info: GodotClassInfo {{
                this: obj,
            }},
"#, name = class.name).unwrap();
        if class.base_class != "" {
            writeln!(output, r#"
            parent: {parent}::from_object(obj),
            "#, parent = class.base_class).unwrap();
        }
        writeln!(output, r#"
        }}
    }}
}}
"#).unwrap();
        if class.base_class != "" {
            writeln!(output, r#"
impl Deref for {name} {{
    type Target = {parent};
    fn deref(&self) -> &Self::Target {{
        &self.parent
    }}
}}
            "#, name = class.name, parent = class.base_class).unwrap();
        }
        writeln!(output, r#"
impl {name} {{"#, name = class.name
        ).unwrap();

        let s_name = if class.name.starts_with("_") {
            &class.name[1..]
        } else {
            class.name.as_ref()
        };

        if class.singleton {
            writeln!(output, r#"
    pub fn godot_singleton() -> GodotRef<{name}> {{
        unsafe {{
            let obj = (get_api().godot_global_get_singleton)(b"{s_name}\0".as_ptr() as *mut _);
            GodotRef::from_raw(obj as *mut _)
        }}
    }}
            "#, name = class.name, s_name = s_name).unwrap();
        }

        if class.instanciable {
            writeln!(output, r#"
    pub fn new() -> GodotRef<Self> {{
        unsafe {{
            let ctor = (get_api().godot_get_class_constructor)(b"{s_name}\0".as_ptr() as *mut _).unwrap();
            let obj = ctor();
            return GodotRef::from_object(obj as *mut _);
        }}
    }}"#, s_name = s_name
            ).unwrap();
        }

        'method:
        for method in class.methods {
            // GDScript and NativeScript have ::new methods but we want to reserve
            // the name for the constructors.
            let method_name = match method.name.as_str() {
                "new" => "_new",
                name => name,
            };

            if method_name == "free" {
                // Awful hack (which the C++ bindings also do)!
                // free is exported but doesn't actually exist and crashes the engine,
                // so use godot_object_destroy instead in GodotRef.
                continue 'method;
            }

            let rust_ret_type = if let Some(ty) = method.get_return_type().to_rust() {
                ty
            } else {
                continue
            };

            let mut type_params = String::new();
            let mut params = String::new();
            for argument in &method.arguments {
                if let Some(ty) = argument.get_type().to_rust() {
                    fmt::Write::write_fmt(&mut params, format_args!(", {}: {}", rust_safe_name(&argument.name), ty)).unwrap();
                } else {
                    continue 'method;
                }
            }

            if method.has_varargs {
                params.push_str(", varargs: &[Variant]");
            }

            writeln!(output, r#"

    pub fn {name}<{type_params}>(&self{params}) -> {rust_ret_type} {{
        unsafe {{
            let api = ::get_api();
            static mut METHOD_BIND: *mut sys::godot_method_bind = 0 as _;
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| {{
                let class = b"{cname}\0".as_ptr();
                let method = b"{name}\0".as_ptr();
                METHOD_BIND = (api.godot_method_bind_get_method)(
                    class as *const _,
                    method as *const _
                );
                debug_assert!(!METHOD_BIND.is_null());
            }});

            "#, cname = class.name, name = method_name, rust_ret_type = rust_ret_type, params = params,
                type_params = type_params).unwrap();
            if method.has_varargs {
                writeln!(output, r#"
            let mut argument_buffer: Vec<*const sys::godot_variant> = Vec::with_capacity({arg_count} + varargs.len());
                "#, arg_count = method.arguments.len()).unwrap();

                for argument in &method.arguments {
                    let ty = argument.get_type().to_rust().unwrap();
                    if ty.starts_with("Option") {
                        writeln!(output, r#"
                let {name}: Variant = if let Some(o) = {name} {{
                    o.into()
                }} else {{ Variant::new() }};
                        "#, name = rust_safe_name(&argument.name)).unwrap();
                    } else if ty == "GodotString" {
                        writeln!(output, r#"
                let {name}: Variant = Variant::from_godot_string(&{name});
                        "#, name = rust_safe_name(&argument.name)).unwrap();
                    } else {
                        writeln!(output, r#"
                let {name}: Variant = {name}.into();
                        "#, name = rust_safe_name(&argument.name)).unwrap();
                    }
                    writeln!(output, r#"
            argument_buffer.push(&{name}.0);
                    "#, name = rust_safe_name(&argument.name)).unwrap();
                }

                writeln!(output, r#"
            for arg in varargs {{
                argument_buffer.push(&arg.0 as *const _);
            }}
            let ret = Variant((api.godot_method_bind_call)(METHOD_BIND, self.info.this, argument_buffer.as_mut_ptr(), argument_buffer.len() as _, ptr::null_mut()));
                "#).unwrap();

                if rust_ret_type.starts_with("Option") {
                    writeln!(output, r#"
                ret.try_to_object()
                    "#).unwrap();
                } else {
                    writeln!(output, r#"
                ret.into()
                    "#).unwrap();
                }

            } else {
                writeln!(output, r#"
            let mut argument_buffer = [ptr::null() as *const libc::c_void; {arg_count}];
                "#, arg_count = method.arguments.len()).unwrap();

                for (idx, argument) in method.arguments.iter().enumerate() {
                    godot_handle_argument_pre(&mut output, &argument.get_type(), rust_safe_name(&argument.name), idx);
                }

                godot_handle_return_pre(&mut output, &method.get_return_type());

                writeln!(output, r#"
            (api.godot_method_bind_ptrcall)(METHOD_BIND, self.info.this, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);
                "#).unwrap();

                godot_handle_return_post(&mut output, &method.get_return_type());
            }

            writeln!(output, r#"
        }}
    }}"#).unwrap();
        }

        writeln!(output, r#"}}"#).unwrap();
    }
}

fn rust_safe_name(name: &str) -> &str {
    match name {
        "use" => "_use",
        "type" => "_type",
        "loop" => "_loop",
        "in" => "_in",
        "override" => "_override",
        "where" => "_where",
        name => name,
    }
}

fn godot_handle_argument_pre<W: Write>(w: &mut W, ty: &Ty, name: &str, arg: usize) {
    match ty {
        &Ty::Bool
        | &Ty::F64
        | &Ty::I64
        | &Ty::Vector2
        | &Ty::Vector3
        | &Ty::Transform
        | &Ty::Transform2D
        | &Ty::Quat
        | &Ty::Plane
        | &Ty::Aabb
        | &Ty::Basis
        | &Ty::Rect2
        | &Ty::Color
        => {
            writeln!(w, r#"
            argument_buffer[{arg}] = (&{name}) as *const _ as *const _;
            "#, name = name, arg = arg).unwrap();
        },
        &Ty::Variant
        | &Ty::String
        | &Ty::Rid
        | &Ty::NodePath
        | &Ty::VariantArray
        | &Ty::Dictionary
        | &Ty::ByteArray
        | &Ty::StringArray
        | &Ty::Vector2Array
        | &Ty::Vector3Array
        | &Ty::ColorArray
        | &Ty::Int32Array
        | &Ty::Float32Array
        => {
            writeln!(w, r#"
            argument_buffer[{arg}] = (&{name}.0) as *const _ as *const _;
            "#, name = name, arg = arg).unwrap();
        },
        &Ty::Object(_) => {
            writeln!(w, r#"
            argument_buffer[{arg}] = if let Some(arg) = {name} {{
                arg.this as *const _ as *const _
            }} else {{
                ptr::null()
            }};
            "#, name = name, arg = arg).unwrap();
        },
        _ => {}
    }
}

fn godot_handle_return_pre<W: Write>(w: &mut W, ty: &Ty) {
    match ty {
        &Ty::Void => {
            writeln!(w, r#"
            let ret_ptr = ptr::null_mut();
            "#).unwrap();

        },
        &Ty::F64 => {
            writeln!(w, r#"
            let mut ret = 0.0f64;
            let ret_ptr = &mut ret as *mut _;
            "#).unwrap();
        },
        &Ty::I64 => {
            writeln!(w, r#"
            let mut ret = 0i64;
            let ret_ptr = &mut ret as *mut _;
            "#).unwrap();
        },
        &Ty::Bool => {
            writeln!(w, r#"
            let mut ret = false;
            let ret_ptr = &mut ret as *mut _;
            "#).unwrap();
        },
        &Ty::String
        | &Ty::Vector2
        | &Ty::Vector3
        | &Ty::Transform
        | &Ty::Transform2D
        | &Ty::Quat
        | &Ty::Plane
        | &Ty::Rect2
        | &Ty::Basis
        | &Ty::Color
        | &Ty::NodePath
        | &Ty::Variant
        | &Ty::Aabb
        | &Ty::VariantArray
        | &Ty::Dictionary
        | &Ty::ByteArray
        | &Ty::StringArray
        | &Ty::Vector2Array
        | &Ty::Vector3Array
        | &Ty::ColorArray
        | &Ty::Int32Array
        | &Ty::Float32Array
        => {
            writeln!(w, r#"
            let mut ret = {sys_ty}::default();
            let ret_ptr = &mut ret as *mut _;
            "#, sys_ty = ty.to_sys().unwrap()
            ).unwrap();
        }
        &Ty::Object(_) // TODO: double check
        | &Ty::Rid => {
            writeln!(w, r#"
            let mut ret: *mut sys::godot_object = ptr::null_mut();
            let ret_ptr = (&mut ret) as *mut _;
            "#).unwrap();
        }
        &Ty::Result => {
            writeln!(w, r#"
            let mut ret: sys::godot_error = sys::godot_error::GODOT_OK;
            let ret_ptr = (&mut ret) as *mut _;
            "#).unwrap();
        }
        &Ty::VariantType => {
            writeln!(w, r#"
            let mut ret: sys::godot_variant_type = sys::godot_variant_type::GODOT_VARIANT_TYPE_NIL;
            let ret_ptr = (&mut ret) as *mut _;
            "#).unwrap();
        }
        &Ty::Enum(_) => {}
    }
}

fn godot_handle_return_post<W: Write>(w: &mut W, ty: &Ty) {
    match ty {
        &Ty::Void => {},
        &Ty::F64
        | &Ty::I64
        | &Ty::Bool
        => { writeln!(w, "ret").unwrap(); }
        &Ty::Vector2
        | &Ty::Vector3
        | &Ty::Transform
        | &Ty::Transform2D
        | &Ty::Quat
        | &Ty::Aabb
        | &Ty::Rect2
        | &Ty::Basis
        | &Ty::Plane
        | &Ty::Color
        => {
            writeln!(w, "::std::mem::transmute(ret)").unwrap();
        },
        &Ty::Rid => {
            writeln!(w, r#"
            let mut rid = Rid::default();
            (api.godot_rid_new_with_resource)(&mut rid.0, ret);
            rid
            "#).unwrap();
        },
        &Ty::String
        | &Ty::NodePath
        | &Ty::VariantArray
        | &Ty::Dictionary
        | &Ty::ByteArray
        | &Ty::StringArray
        | &Ty::Vector2Array
        | &Ty::Vector3Array
        | &Ty::ColorArray
        | &Ty::Int32Array
        | &Ty::Float32Array
        | &Ty::Variant
        => {
            writeln!(w, r#"
            {rust_ty}(ret)
            "#, rust_ty = ty.to_rust().unwrap()
            ).unwrap();
        }
        &Ty::Object(ref name) => {
            writeln!(w, r#"
            if ret.is_null() {{
                None
            }} else {{
                Some(GodotRef::<{}>::from_object(ret))
            }}
            "#, name).unwrap();
        },
        &Ty::Result => {
            writeln!(w, r#"
            result_from_sys(ret)
            "#).unwrap();
        }
        &Ty::VariantType => {
            writeln!(w, r#"
            VariantType::from_sys(ret)
            "#).unwrap();
        }
        _ => {}
    }
}

#[derive(Deserialize, Debug)]
struct GodotClass {
    name: String,
    base_class: String,
    singleton: bool,
    is_reference: bool,
    instanciable: bool,

    methods: Vec<GodotMethod>,
    enums: Vec<Enum>,
}

//impl GodotClass {
//    fn get_type(&self) -> Ty {
//        Ty::from_src(&self.name)
//    }
//}

#[derive(Deserialize, Debug)]
struct Enum {
    name: String,
    values: HashMap<String, u32>,
}

#[derive(Deserialize, Debug)]
struct GodotMethod {
    name: String,
    return_type: String,

    is_editor: bool,
    is_noscript: bool,
    is_const: bool,
    is_reverse: bool,
    is_virtual: bool,
    has_varargs: bool,

    arguments: Vec<GodotArgument>,
}

impl GodotMethod {
    fn get_return_type(&self) -> Ty {
        Ty::from_src(&self.return_type)
    }
}

#[derive(Deserialize, Debug)]
struct GodotArgument {
    name: String,
    #[serde(rename = "type")]
    ty: String,
    has_default_value: bool,
    default_value: String,
}

impl GodotArgument {
    fn get_type(&self) -> Ty {
        Ty::from_src(&self.ty)
    }
}

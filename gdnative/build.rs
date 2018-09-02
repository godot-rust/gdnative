
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate heck;

use heck::{CamelCase, SnakeCase};

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
            &Ty::Object(ref name) => Some(format!("Option<{}>", name)),
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

    writeln!(output, "use std::os::raw::c_char;").unwrap();
    writeln!(output, "use std::ptr;").unwrap();
    writeln!(output, "use std::mem;").unwrap();
    writeln!(output, "use object;").unwrap();

    for class in classes {
        let has_parent = class.base_class != "";
        let singleton_str = if class.singleton { "singleton " } else { "" } ;
        let ownership_type = if class.is_reference { "reference counted" } else { "manually managed" };
        if has_parent {
            writeln!(output, r#"
/// `{api_type} {singleton}class {name} : {base_class}` ({ownership_type})"#,
                api_type = class.api_type,
                name = class.name,
                base_class = class.base_class,
                ownership_type = ownership_type,
                singleton = singleton_str
            ).unwrap();
        } else {
            writeln!(output, r#"
/// `{api_type} {singleton}class {name}` ({ownership_type})"#,
                api_type = class.api_type,
                name = class.name,
                ownership_type = ownership_type,
                singleton = singleton_str
            ).unwrap();
        }

        if class.base_class != "" {
            writeln!(output,
r#"///
/// ## Base class
///
/// {name} inherits [{base_class}](struct.{base_class}.html) and all of its methods."#,
                name = class.name,
                base_class = class.base_class
            ).unwrap();
        }

        if class.is_reference {
            writeln!(output,
r#"///
/// ## Memory management
///
/// The lifetime of this object is automatically managed through reference counting."#
            ).unwrap();
        } else if class.instanciable {
            writeln!(output,
r#"///
/// ## Memory management
///
/// Non reference counted objects such as the ones of this type are usually owned by the engine.
/// In the cases where Rust code owns an object of this type, ownership should be either passed
/// to the engine or the object must be manually destroyed using `{}::free`."#,
                class.name
            ).unwrap();
        }

        if class.api_type == "tools" {
            writeln!(output,
r#"///
/// ## Tool
///
/// This class is used to interact with godot's editor."#,
            ).unwrap();
        }

        writeln!(output,
r#"#[allow(non_camel_case_types)]
pub struct {name} {{
    this: *mut sys::godot_object,
}}
"#,
            name = class.name
        ).unwrap();

        for e in &class.enums {
            // TODO: check whether the start of the variant name is
            // equal to the end of the enum name and if so don't repeat it
            // it. For example ImageFormat::Rgb8 instead of ImageFormat::FormatRgb8.
            writeln!(output, r#"
#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum {class_name}{enum_name} {{
"#, class_name = class.name, enum_name = e.name
            ).unwrap();

            for (key, val) in &e.values {
                let key = key.as_str().to_camel_case();
                writeln!(output,
r#"    {key} = {val},"#,
                    key = key,
                    val = val
                ).unwrap();
            }
            writeln!(output, r#"
}}"#
            ).unwrap();
        }

        writeln!(output, r#"
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct {name}MethodTable {{
    pub class_constructor: sys::godot_class_constructor,"#,
            name = class.name
        ).unwrap();

        for method in &class.methods {
            let method_name = method.get_name();
            if skip_method(&method_name) {
                continue;
            }
            writeln!(output, "    pub {}: *mut sys::godot_method_bind,", method_name).unwrap();
        }
        writeln!(output, r#"
}}

impl {name}MethodTable {{
    unsafe fn get_mut() -> &'static mut Self {{
        static mut TABLE: {name}MethodTable = {name}MethodTable {{
            class_constructor: None,"#,
            name = class.name
        ).unwrap();
        for method in &class.methods {
            let method_name = method.get_name();
            if skip_method(&method_name) {
                continue;
            }
            writeln!(output,
"            {}: 0 as *mut sys::godot_method_bind,",
                method.get_name()
            ).unwrap();
        }
        writeln!(output, r#"
        }};

        &mut TABLE
    }}

    pub unsafe fn unchecked_get() -> &'static Self {{
        Self::get_mut()
    }}

    pub fn get(gd_api: &GodotApi) -> &'static Self {{
        unsafe {{
            let table = Self::get_mut();
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| {{
                {name}MethodTable::init(table, gd_api);
            }});

            table
        }}
    }}

    #[inline(never)]
    fn init(table: &mut Self, gd_api: &GodotApi) {{
        unsafe {{
            let class_name = b"{name}\0".as_ptr() as *const c_char;
            table.class_constructor = (gd_api.godot_get_class_constructor)(class_name);"#,
                name = class.name
            ).unwrap();
        for method in &class.methods {
            let method_name = method.get_name();
            if skip_method(&method_name) {
                continue;
            }

            writeln!(output,
r#"            table.{method_name} = (gd_api.godot_method_bind_get_method)(class_name, "{method_name}\0".as_ptr() as *const c_char );"#,
                method_name = method_name
            ).unwrap();
        }

        writeln!(output, r#"
        }}
    }}
}}"#
        ).unwrap();


        writeln!(output, r#"

unsafe impl GodotObject for {name} {{
    fn class_name() -> &'static str {{
        "{name}"
    }}

    unsafe fn from_sys(obj: *mut sys::godot_object) -> Self {{
        {addref_if_reference}
        Self {{ this: obj, }}
    }}

    unsafe fn to_sys(&self) -> *mut sys::godot_object {{
        self.this
    }}
}}

"#,
            name = class.name,
            addref_if_reference = if class.is_reference { "object::add_ref(obj);" } else { "" }
        ).unwrap();

        if class.base_class != "" {
            writeln!(output, r#"
impl Deref for {name} {{
    type Target = {parent};
    fn deref(&self) -> &Self::Target {{
        unsafe {{
            mem::transmute(self)
        }}
    }}
}}

impl DerefMut for {name} {{
    fn deref_mut(&mut self) -> &mut Self::Target {{
        unsafe {{
            mem::transmute(self)
        }}
    }}
}}
"#,             name = class.name,
                parent = class.base_class
            ).unwrap();
        }


        'method_def:
        for method in &class.methods {
            let method_name = method.get_name();

            if skip_method(&method_name) {
                continue;
            }

            let rust_ret_type = if let Some(ty) = method.get_return_type().to_rust() {
                ty
            } else {
                continue;
            };

            let mut params = String::new();
            for argument in &method.arguments {
                if let Some(ty) = argument.get_type().to_rust() {
                    fmt::Write::write_fmt(&mut params, format_args!(", {}: {}", rust_safe_name(&argument.name), ty)).unwrap();
                } else {
                    continue 'method_def;
                }
            }

            if method.has_varargs {
                params.push_str(", varargs: &[Variant]");
            }

            writeln!(output, r#"

unsafe fn {cname}_{name}(obj_ptr: *mut sys::godot_object{params}) -> {rust_ret_type} {{
    let gd_api = ::get_api();

    let method_bind: *mut sys::godot_method_bind = {cname}MethodTable::get(gd_api).{name};"#,
                cname = class.name,
                name = method_name,
                rust_ret_type = rust_ret_type,
                params = params,
            ).unwrap();
            if method.has_varargs {
                writeln!(output,
r#"    let mut argument_buffer: Vec<*const sys::godot_variant> = Vec::with_capacity({arg_count} + varargs.len());"#,
                    arg_count = method.arguments.len()
                ).unwrap();

                for argument in &method.arguments {
                    let ty = argument.get_type().to_rust().unwrap();
                    if ty.starts_with("Option") {
                        writeln!(output,
r#"    let {name}: Variant = if let Some(o) = {name} {{
           o.into()
       }} else {{ Variant::new() }};"#,
                            name = rust_safe_name(&argument.name)
                        ).unwrap();
                    } else if ty == "GodotString" {
                        writeln!(output,
r#"    let {name}: Variant = Variant::from_godot_string(&{name});"#,
                            name = rust_safe_name(&argument.name)
                        ).unwrap();
                    } else {
                        writeln!(output, r#"
       let {name}: Variant = {name}.into();"#,
                            name = rust_safe_name(&argument.name)
                        ).unwrap();
                    }
                    writeln!(output,
r#"    argument_buffer.push(&{name}.0); "#,
                        name = rust_safe_name(&argument.name)
                    ).unwrap();
                }

                writeln!(output, r#"
    for arg in varargs {{
        argument_buffer.push(&arg.0 as *const _);
    }}
    let ret = Variant((gd_api.godot_method_bind_call)(method_bind, obj_ptr, argument_buffer.as_mut_ptr(), argument_buffer.len() as _, ptr::null_mut()));"#
                ).unwrap();

                if rust_ret_type.starts_with("Option") {
                    writeln!(output,
r#"    ret.try_to_object()"#
                    ).unwrap();
                } else {
                    writeln!(output,
r#"    ret.into()"#
                    ).unwrap();
                }

            } else {
                writeln!(output, r#"
    let mut argument_buffer = [ptr::null() as *const libc::c_void; {arg_count}];"#,
                    arg_count = method.arguments.len()).unwrap();

                for (idx, argument) in method.arguments.iter().enumerate() {
                    godot_handle_argument_pre(&mut output, &argument.get_type(), rust_safe_name(&argument.name), idx);
                }

                godot_handle_return_pre(&mut output, &method.get_return_type());

                writeln!(output, r#"
    (gd_api.godot_method_bind_ptrcall)(method_bind, obj_ptr, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);"#
                ).unwrap();

                godot_handle_return_post(&mut output, &method.get_return_type());
            }

            writeln!(output,
r#"}}"#
            ).unwrap();
        }


        writeln!(output, r#"
impl {name} {{"#, name = class.name
        ).unwrap();

        let s_name = if class.name.starts_with("_") {
            &class.name[1..]
        } else {
            class.name.as_ref()
        };

        if class.base_class != "" {
            writeln!(output, r#"
    /// Up-cast.
    pub fn as_{parent_sc}(&self) -> {parent} {{
        unsafe {{ {parent}::from_sys(self.this) }}
    }}"#,
                parent = class.base_class,
                parent_sc = class_name_to_snake_case(&class.base_class)
            ).unwrap();
        }

        if class.singleton {
            writeln!(output, r#"
    pub fn godot_singleton() -> Self {{
        unsafe {{
            let this = (get_api().godot_global_get_singleton)(b"{s_name}\0".as_ptr() as *mut _);

            {name} {{
                this
            }}
        }}
    }}
            "#, name = class.name, s_name = s_name).unwrap();
        }

        if class.instanciable {

            if class.is_reference {
                writeln!(output,
r#"
    // Constructor
    pub fn new() -> Self {{
        unsafe {{
            let gd_api = ::get_api();
            let ctor = {name}MethodTable::get(gd_api).class_constructor.unwrap();
            let obj = ctor();
            object::init_ref_count(obj);

            {name} {{
                this: obj
            }}
        }}
    }}

    /// Creates a new reference to the same object.
    pub fn new_ref(&self) -> Self {{
        unsafe {{
            object::add_ref(self.this);

            Self {{
                this: self.this,
            }}
        }}
    }}
"#,
                    name = class.name
                ).unwrap();
            } else {

                writeln!(output,
r#"
    /// Constructor.
    ///
    /// Because this type is not reference counted, the lifetime of the returned object
    /// is *not* automatically managed.
    /// Immediately after creation, the object is owned by the caller, and can be
    /// passed to the engine (in which case the engine will be responsible for
    /// destroying the object) or destroyed manually using `{name}::free`.
    pub fn new() -> Self {{
        unsafe {{
            let gd_api = ::get_api();
            let ctor = {name}MethodTable::get(gd_api).class_constructor.unwrap();
            let this = ctor();

            {name} {{
                this
            }}
        }}
    }}

    /// Manually deallocate the object.
    pub unsafe fn free(self) {{
        (get_api().godot_object_destroy)(self.this);
    }}
"#,
                    name = class.name
                ).unwrap();
            }
        }

        'method:
        for method in class.methods {
            let method_name = method.get_name();

            if skip_method(&method_name) {
                continue 'method;
            }

            let rust_ret_type = if let Some(ty) = method.get_return_type().to_rust() {
                ty
            } else {
                continue
            };

            let mut params_decl = String::new();
            let mut params_use = String::new();
            for argument in &method.arguments {
                if let Some(ty) = argument.get_type().to_rust() {
                    fmt::Write::write_fmt(&mut params_decl, format_args!(", {}: {}", rust_safe_name(&argument.name), ty)).unwrap();
                    fmt::Write::write_fmt(&mut params_use, format_args!(", {}", rust_safe_name(&argument.name))).unwrap();
                } else {
                    continue 'method;
                }
            }

            if method.has_varargs {
                params_decl.push_str(", varargs: &[Variant]");
                params_use.push_str(", varargs");
            }

            let self_param = if method.is_const { "&self" } else { "&mut self" };

            writeln!(output, r#"

    pub fn {name}({self_param}{params_decl}) -> {rust_ret_type} {{
        unsafe {{
            {cname}_{name}(self.this{params_use})
        }}
    }}"#,
                cname = class.name,
                name = method_name,
                rust_ret_type = rust_ret_type,
                params_decl = params_decl,
                params_use = params_use,
                self_param = self_param,
            ).unwrap();
        }

        writeln!(output,
r#"
    pub fn cast<T: GodotObject>(&self) -> Option<T> {{
        object::godot_cast::<T>(self.this)
    }}
"#      ).unwrap();

        writeln!(output, r#"}}"#).unwrap();

        if class.is_reference && class.instanciable {
            writeln!(output,
r#"
impl Drop for {name} {{
    fn drop(&mut self) {{
        unsafe {{
            if object::unref(self.this) {{
                (::get_api().godot_object_destroy)(self.this);
            }}
        }}
    }}
}}
"#,
                name = class.name
            ).unwrap();
        }
    }
}

fn skip_method(name: &str) -> bool {
    name == "free"
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
            writeln!(w,
r#"    argument_buffer[{arg}] = (&{name}) as *const _ as *const _;"#,
            name = name, arg = arg).unwrap();
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
            writeln!(w,
r#"    argument_buffer[{arg}] = (&{name}.0) as *const _ as *const _;"#,
                name = name, arg = arg
            ).unwrap();
        },
        &Ty::Object(_) => {
            writeln!(w, r#"
    argument_buffer[{arg}] = if let Some(arg) = {name} {{
        arg.this as *const _ as *const _
    }} else {{
        ptr::null()
    }};"#,
                name = name,
                arg = arg
            ).unwrap();
        },
        _ => {}
    }
}

fn godot_handle_return_pre<W: Write>(w: &mut W, ty: &Ty) {
    match ty {
        &Ty::Void => {
            writeln!(w, r#"
    let ret_ptr = ptr::null_mut();"#).unwrap();
        },
        &Ty::F64 => {
            writeln!(w, r#"
    let mut ret = 0.0f64;
    let ret_ptr = &mut ret as *mut _;"#
            ).unwrap();
        },
        &Ty::I64 => {
            writeln!(w, r#"
    let mut ret = 0i64;
    let ret_ptr = &mut ret as *mut _;"#
            ).unwrap();
        },
        &Ty::Bool => {
            writeln!(w, r#"
    let mut ret = false;
    let ret_ptr = &mut ret as *mut _;"#
            ).unwrap();
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
    let ret_ptr = &mut ret as *mut _;"#,
                sys_ty = ty.to_sys().unwrap()
            ).unwrap();
        }
        &Ty::Object(_) // TODO: double check
        | &Ty::Rid => {
            writeln!(w, r#"
    let mut ret: *mut sys::godot_object = ptr::null_mut();
    let ret_ptr = (&mut ret) as *mut _;"#
            ).unwrap();
        }
        &Ty::Result => {
            writeln!(w, r#"
    let mut ret: sys::godot_error = sys::godot_error::GODOT_OK;
    let ret_ptr = (&mut ret) as *mut _;"#
            ).unwrap();
        }
        &Ty::VariantType => {
            writeln!(w, r#"
    let mut ret: sys::godot_variant_type = sys::godot_variant_type::GODOT_VARIANT_TYPE_NIL;
    let ret_ptr = (&mut ret) as *mut _;"#
            ).unwrap();
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
        => {
            writeln!(w, r#"    ret"#).unwrap();
        }
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
            writeln!(w, r#"    mem::transmute(ret)"#).unwrap();
        },
        &Ty::Rid => {
            writeln!(w, r#"
    let mut rid = Rid::default();
    (gd_api.godot_rid_new_with_resource)(&mut rid.0, ret);
    rid"#
            ).unwrap();
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
            writeln!(w,
r#"    {rust_ty}(ret)"#, rust_ty = ty.to_rust().unwrap()
            ).unwrap();
        }
        &Ty::Object(ref name) => {
            writeln!(w, r#"
    if ret.is_null() {{
        None
    }} else {{
        Some({}::from_sys(ret))
    }}"#,
                name
            ).unwrap();
        },
        &Ty::Result => {
            writeln!(w, r#"    result_from_sys(ret)"#).unwrap();
        }
        &Ty::VariantType => {
            writeln!(w, r#"    VariantType::from_sys(ret)"#).unwrap();
        }
        _ => {}
    }
}

#[derive(Deserialize, Debug)]
struct GodotClass {
    name: String,
    base_class: String,
    api_type: String,
    singleton: bool,
    is_reference: bool,
    instanciable: bool,

    methods: Vec<GodotMethod>,
    enums: Vec<Enum>,
}

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
    fn get_name(&self) -> &str {
        // GDScript and NativeScript have ::new methods but we want to reserve
        // the name for the constructors.
        if &self.name == "new" {
            return "_new";
        }

        &self.name
    }

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

fn class_name_to_snake_case(name: &str) -> String {
    // TODO: this is a quick-n-dirty band-aid, it'd be better to
    // programmatically do the right conversion, but to_snake_case
    // currently translates "Node2D" into "node2_d".
    match name {
        "SpriteBase3D" => "sprite_base_3d".to_string(),
        "Node2D" => "node_2d".to_string(),
        "CollisionObject2D" => "collision_object_2d".to_string(),
        "PhysicsBody2D" => "physics_body_2d".to_string(),
        "VisibilityNotifier2D" => "visibility_notifier_2d".to_string(),
        "Joint2D" => "joint_2d".to_string(),
        "Shape2D" => "shape_2d".to_string(),
        "Physics2DServer" => "physics_2d_server".to_string(),
        "Physics2DDirectBodyState" => "physics_2d_direct_body_state".to_string(),
        _ => name.to_snake_case(),
    }
}

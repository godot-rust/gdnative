use crate::api::*;
use crate::documentation::class_doc_link;
use crate::rust_safe_name;
use crate::GeneratorResult;

use std::collections::HashSet;
use std::fmt;
use std::io::Write;

fn skip_method(name: &str) -> bool {
    name == "free" || name == "reference" || name == "unreference"
}

pub fn generate_method_table(
    output: &mut impl Write,
    class: &GodotClass,
    has_underscore: bool,
) -> GeneratorResult {
    writeln!(
        output,
        r#"
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct {name}MethodTable {{
    pub class_constructor: sys::godot_class_constructor,"#,
        name = class.name
    )?;

    for method in &class.methods {
        let method_name = method.get_name();
        if method_name == "free" {
            continue;
        }
        writeln!(
            output,
            "    pub {}: *mut sys::godot_method_bind,",
            method_name
        )?;
    }
    writeln!(
        output,
        r#"
}}

impl {name}MethodTable {{
    unsafe fn get_mut() -> &'static mut Self {{
        static mut TABLE: {name}MethodTable = {name}MethodTable {{
            class_constructor: None,"#,
        name = class.name
    )?;
    for method in &class.methods {
        let method_name = method.get_name();
        if method_name == "free" {
            continue;
        }
        writeln!(
            output,
            "            {}: 0 as *mut sys::godot_method_bind,",
            method.get_name()
        )?;
    }
    let lookup_name: String = if has_underscore {
        format!("_{class}", class = class.name)
    } else {
        class.name.clone()
    };
    writeln!(
        output,
        r#"
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
            let class_name = b"{lookup_name}\0".as_ptr() as *const c_char;
            table.class_constructor = (gd_api.godot_get_class_constructor)(class_name);"#,
        name = class.name,
        lookup_name = lookup_name,
    )?;
    for method in &class.methods {
        let method_name = method.get_name();
        if method_name == "free" {
            continue;
        }

        writeln!(output,
r#"            table.{method_name} = (gd_api.godot_method_bind_get_method)(class_name, "{method_name}\0".as_ptr() as *const c_char );"#,
            method_name = method_name
        )?;
    }

    writeln!(
        output,
        r#"
        }}
    }}
}}"#
    )?;

    Ok(())
}

pub fn generate_method_impl(
    output: &mut impl Write,
    class: &GodotClass,
    method: &GodotMethod,
) -> GeneratorResult {
    let method_name = method.get_name();

    if skip_method(&method_name) {
        return Ok(());
    }

    let rust_ret_type = if let Some(ty) = method.get_return_type().to_rust() {
        ty
    } else {
        writeln!(output, "// TODO: missing method {}", method_name)?;
        return Ok(());
    };

    let mut params = String::new();
    for argument in &method.arguments {
        if let Some(ty) = argument.get_type().to_rust() {
            fmt::Write::write_fmt(
                &mut params,
                format_args!(", {}: {}", rust_safe_name(&argument.name), ty),
            )
            .unwrap();
        } else {
            writeln!(output, "// TODO: missing method {}", method_name)?;
            return Ok(());
        }
    }

    if method.has_varargs {
        params.push_str(", varargs: &[Variant]");
    }

    writeln!(
        output,
        r#"

#[doc(hidden)]
pub unsafe fn {cname}_{name}(obj_ptr: *mut sys::godot_object{params}) -> {rust_ret_type} {{
    let gd_api = get_api();

    let method_bind: *mut sys::godot_method_bind = {cname}MethodTable::get(gd_api).{name};"#,
        cname = class.name,
        name = method_name,
        rust_ret_type = rust_ret_type,
        params = params,
    )?;
    if method.has_varargs {
        writeln!(output,
r#"    let mut argument_buffer: Vec<*const sys::godot_variant> = Vec::with_capacity({arg_count} + varargs.len());"#,
            arg_count = method.arguments.len()
        )?;

        for argument in &method.arguments {
            let ty = argument.get_type().to_rust().unwrap();
            if ty.starts_with("Option") {
                writeln!(
                    output,
                    r#"    let {name}: Variant = if let Some(o) = {name} {{
           o.into()
       }} else {{ Variant::new() }};"#,
                    name = rust_safe_name(&argument.name)
                )?;
            } else if ty == "GodotString" {
                writeln!(
                    output,
                    r#"    let {name}: Variant = Variant::from_godot_string(&{name});"#,
                    name = rust_safe_name(&argument.name)
                )?;
            } else {
                writeln!(
                    output,
                    r#"
       let {name}: Variant = {name}.into();"#,
                    name = rust_safe_name(&argument.name)
                )?;
            }
            writeln!(
                output,
                r#"    argument_buffer.push({name}.sys()); "#,
                name = rust_safe_name(&argument.name)
            )?;
        }

        writeln!(output, r#"
    for arg in varargs {{
        argument_buffer.push(arg.sys() as *const _);
    }}
    let ret = Variant::from_sys((gd_api.godot_method_bind_call)(method_bind, obj_ptr, argument_buffer.as_mut_ptr(), argument_buffer.len() as _, ptr::null_mut()));"#
        )?;

        if rust_ret_type.starts_with("Option") {
            writeln!(output, r#"    ret.try_to_object()"#)?;
        } else {
            writeln!(output, r#"    ret.into()"#)?;
        }
    } else {
        writeln!(
            output,
            r#"
    let mut argument_buffer : [*const libc::c_void; {arg_count}] = ["#,
            arg_count = method.arguments.len()
        )?;

        for argument in &method.arguments {
            generate_argument_pre(output, &argument.get_type(), rust_safe_name(&argument.name))?;
        }
        writeln!(
            output,
            r#"
    ];"#
        )?;

        generate_return_pre(output, &method.get_return_type())?;

        writeln!(output, r#"
    (gd_api.godot_method_bind_ptrcall)(method_bind, obj_ptr, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);"#
        )?;

        generate_return_post(output, &method.get_return_type())?;
    }

    writeln!(output, r#"}}"#)?;

    Ok(())
}

pub fn generate_methods(
    output: &mut impl Write,
    api: &Api,
    method_set: &mut HashSet<String>,
    class_name: &str,
    is_safe: bool,
    is_leaf: bool,
) -> GeneratorResult {
    if let Some(class) = api.find_class(class_name) {
        'method: for method in &class.methods {
            let method_name = method.get_name();

            if skip_method(&method_name) {
                continue;
            }

            let rust_ret_type = if let Some(ty) = method.get_return_type().to_rust() {
                ty
            } else {
                continue;
            };

            // Ensure that methods are not injected several times.
            let method_name_string = method_name.to_string();
            if method_set.contains(&method_name_string) {
                continue;
            }
            method_set.insert(method_name_string);

            let mut params_decl = String::new();
            let mut params_use = String::new();
            for argument in &method.arguments {
                if let Some(ty) = argument.get_type().to_rust() {
                    fmt::Write::write_fmt(
                        &mut params_decl,
                        format_args!(", {}: {}", rust_safe_name(&argument.name), ty),
                    )
                    .unwrap();
                    fmt::Write::write_fmt(
                        &mut params_use,
                        format_args!(", {}", rust_safe_name(&argument.name)),
                    )
                    .unwrap();
                } else {
                    continue 'method;
                }
            }

            if method.has_varargs {
                params_decl.push_str(", varargs: &[Variant]");
                params_use.push_str(", varargs");
            }

            let self_param = if method.is_const {
                "&self"
            } else {
                "&mut self"
            };

            if !is_leaf {
                writeln!(output, "    /// Inherited from {}.", class_doc_link(class))?;
            }

            //let namespace = format!("gdnative_{:?}_private::", api.namespaces[&class.name]);
            let namespace = "";

            if is_safe {
                writeln!(
                    output,
                    r#"    #[inline]
    pub fn {name}({self_param}{params_decl}) -> {rust_ret_type} {{
        unsafe {{ {namespace}{cname}_{name}(self.this{params_use}) }}
    }}
"#,
                    cname = class.name,
                    name = method_name,
                    namespace = namespace,
                    rust_ret_type = rust_ret_type,
                    params_decl = params_decl,
                    params_use = params_use,
                    self_param = self_param,
                )?;
            } else {
                writeln!(
                    output,
                    r#"    #[inline]
    pub unsafe fn {name}({self_param}{params_decl}) -> {rust_ret_type} {{
        {namespace}{cname}_{name}(self.this{params_use})
    }}
"#,
                    cname = class.name,
                    name = method_name,
                    namespace = namespace,
                    rust_ret_type = rust_ret_type,
                    params_decl = params_decl,
                    params_use = params_use,
                    self_param = self_param,
                )?;
            }
        }

        // Reference includes all of Object's methods so they are safe.
        if class.base_class == "Reference" {
            generate_methods(output, api, method_set, &class.base_class, is_safe, false)?;
        }
    }
    Ok(())
}

fn generate_argument_pre(w: &mut impl Write, ty: &Ty, name: &str) -> GeneratorResult {
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
        | &Ty::Color => {
            writeln!(
                w,
                r#"        (&{name}) as *const _ as *const _,"#,
                name = name
            )?;
        }
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
        | &Ty::Float32Array => {
            writeln!(
                w,
                r#"        {name}.sys() as *const _ as *const _,"#,
                name = name
            )?;
        }
        &Ty::Object(_) => {
            writeln!(w, r#"        if let Some(arg) = {name} {{ arg.this as *const _ as *const _ }} else {{ ptr::null() }},"#,
                name = name,
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn generate_return_pre(w: &mut impl Write, ty: &Ty) -> GeneratorResult {
    match ty {
        &Ty::Void => {
            writeln!(w, r#"
    let ret_ptr = ptr::null_mut();"#)?;
        },
        &Ty::F64 => {
            writeln!(w, r#"
    let mut ret = 0.0f64;
    let ret_ptr = &mut ret as *mut _;"#
            )?;
        },
        &Ty::I64 => {
            writeln!(w, r#"
    let mut ret = 0i64;
    let ret_ptr = &mut ret as *mut _;"#
            )?;
        },
        &Ty::Bool => {
            writeln!(w, r#"
    let mut ret = false;
    let ret_ptr = &mut ret as *mut _;"#
            )?;
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
        | &Ty::Rid
        => {
            writeln!(w, r#"
    let mut ret = {sys_ty}::default();
    let ret_ptr = &mut ret as *mut _;"#,
                sys_ty = ty.to_sys().unwrap()
            )?;
        }
        &Ty::Object(_) // TODO: double check
        => {
            writeln!(w, r#"
    let mut ret: *mut sys::godot_object = ptr::null_mut();
    let ret_ptr = (&mut ret) as *mut _;"#
            )?;
        }
        &Ty::Result => {
            writeln!(w, r#"
    let mut ret: sys::godot_error = sys::godot_error_GODOT_OK;
    let ret_ptr = (&mut ret) as *mut _;"#
            )?;
        }
        &Ty::VariantType => {
            writeln!(w, r#"
    let mut ret: sys::godot_variant_type = sys::godot_variant_type_GODOT_VARIANT_TYPE_NIL;
    let ret_ptr = (&mut ret) as *mut _;"#
            )?;
        }
        &Ty::Enum(ref name) => {
            writeln!(w, r#"
    let mut ret: {} = mem::transmute(0);
    let ret_ptr = (&mut ret) as *mut _;"#,
                name
            )?;
        }
    }

    Ok(())
}

fn generate_return_post(w: &mut impl Write, ty: &Ty) -> GeneratorResult {
    match ty {
        &Ty::Void => {}
        &Ty::F64 | &Ty::I64 | &Ty::Bool => {
            writeln!(
                w,
                r#"
    ret"#
            )?;
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
        | &Ty::Color => {
            writeln!(w, r#"    mem::transmute(ret)"#)?;
        }
        &Ty::Rid => {
            writeln!(
                w,
                r#"    Rid::from_sys(ret)"#
            )?;
        }
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
        | &Ty::Variant => {
            writeln!(
                w,
                r#"
    {rust_ty}::from_sys(ret)"#,
                rust_ty = ty.to_rust().unwrap()
            )?;
        }
        &Ty::Object(ref name) => {
            writeln!(
                w,
                r#"
    if ret.is_null() {{
        None
    }} else {{
        Some({}::from_sys(ret))
    }}"#,
                name
            )?;
        }
        &Ty::Result => {
            writeln!(
                w,
                r#"
    result_from_sys(ret)"#
            )?;
        }
        &Ty::Enum(_) => {
            writeln!(
                w,
                r#"
    ret"#,
            )?;
        }
        &Ty::VariantType => {
            writeln!(
                w,
                r#"
    VariantType::from_sys(ret)"#
            )?;
        }
    }

    Ok(())
}

use crate::api::*;
use crate::class_docs::GodotXmlDocs;
use crate::rust_safe_name;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::collections::HashMap;

/// Types of icalls.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum IcallType {
    #[cfg(feature = "ptrcall")]
    Ptr,
    Varargs,
    Var,
}

// The `return_type` field is currently only used when ptrcall is enabled
#[cfg_attr(not(feature = "ptrcall"), allow(dead_code))]
pub(crate) struct MethodSig {
    pub(crate) return_type: Ty,
    pub(crate) arguments: Vec<Ty>,
    pub(crate) has_varargs: bool,
}

impl MethodSig {
    pub(crate) fn from_method(method: &GodotMethod) -> Self {
        // reduce the amoun of types so that more icalls can be shared.
        fn ty_erase(ty: Ty) -> Ty {
            match ty {
                Ty::Vector3Axis
                | Ty::Result
                | Ty::VariantType
                | Ty::VariantOperator
                | Ty::Enum(_) => Ty::I64,

                // Objects are erased too, but their path is never inspected.
                Ty::Object(path) => Ty::Object(path),

                other => other,
            }
        }

        let has_varargs = method.has_varargs;

        let return_type = if has_varargs {
            Ty::Variant
        } else {
            Ty::from_src(&method.return_type)
        };

        let mut args = Vec::new();
        for arg in &method.arguments {
            args.push(ty_erase(arg.get_type()));
        }

        Self {
            return_type: ty_erase(return_type),
            arguments: args,
            has_varargs: method.has_varargs,
        }
    }

    pub(crate) fn function_name(&self) -> String {
        // the name for a type used in the name of the icall
        fn ty_arg_name(ty: &Ty) -> &'static str {
            match ty {
                Ty::Void => "void",
                Ty::String => "str",
                Ty::F64 => "f64",
                Ty::I64 => "i64",
                Ty::Bool => "bool",
                Ty::Vector2 => "vec2",
                Ty::Vector3 => "vec3",

                Ty::Quat => "quat",
                Ty::Transform => "trans",
                Ty::Transform2D => "trans2D",
                Ty::Rect2 => "rect2",
                Ty::Plane => "plane",
                Ty::Basis => "basis",
                Ty::Color => "color",
                Ty::NodePath => "nodepath",
                Ty::Variant => "var",
                Ty::Aabb => "aabb",
                Ty::Rid => "rid",
                Ty::VariantArray => "arr",
                Ty::Dictionary => "dict",
                Ty::ByteArray => "bytearr",
                Ty::StringArray => "strarr",
                Ty::Vector2Array => "vec2arr",
                Ty::Vector3Array => "vec3arr",
                Ty::ColorArray => "colorarr",
                Ty::Int32Array => "i32arr",
                Ty::Float32Array => "f32arr",

                Ty::Result
                | Ty::Vector3Axis
                | Ty::VariantType
                | Ty::VariantOperator
                | Ty::Enum(_) => "i64",

                Ty::Object(_) => "obj",
            }
        }

        let icall_ty = self.icall_type();

        // Only ptrcalls have "static" typing on their return types.
        // The other calling types always return `Variant`.
        let mut name = match icall_ty {
            #[cfg(feature = "ptrcall")]
            IcallType::Ptr => format!("icallptr_{}", ty_arg_name(&self.return_type)),
            IcallType::Varargs => String::from("icallvarargs_"),
            IcallType::Var => String::from("icallvar_"),
        };

        for arg in &self.arguments {
            name.push('_');
            name.push_str(ty_arg_name(arg));
        }

        name
    }

    #[allow(clippy::single_match)]
    #[cfg(feature = "ptrcall")]
    pub(crate) fn icall_type(&self) -> IcallType {
        if self.has_varargs {
            return IcallType::Varargs;
        }

        match self.return_type {
            Ty::VariantArray => return IcallType::Var,
            _ => {}
        }

        IcallType::Ptr
    }

    #[cfg(not(feature = "ptrcall"))]
    pub(crate) fn icall_type(&self) -> IcallType {
        if self.has_varargs {
            return IcallType::Varargs;
        }

        IcallType::Var
    }
}

fn skip_method(method: &GodotMethod, name: &str) -> bool {
    const METHODS: &[&str] = &["free", "reference", "unreference", "init_ref"];
    METHODS.contains(&name) || method.is_virtual
}

pub fn generate_method_table(api: &Api, class: &GodotClass) -> TokenStream {
    let has_underscore = api.api_underscore.contains(&class.name);

    let method_table = format_ident!("{}MethodTable", class.name);
    let lookup_name: String = if has_underscore {
        format!("_{class}\0", class = class.name)
    } else {
        format!("{}\0", class.name)
    };

    let struct_methods = class.methods.iter().filter_map(|m| {
        let rust_name = m.get_name().rust_name;
        let rust_ident = format_ident!("{}", rust_name);
        if !skip_method(m, rust_name) {
            Some(quote! { pub #rust_ident: *mut sys::godot_method_bind })
        } else {
            None
        }
    });

    let struct_definition = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        pub(crate) struct #method_table {
            pub class_constructor: sys::godot_class_constructor,
            #(#struct_methods),*
        }
    };

    let impl_methods = class.methods.iter().filter_map(|m| {
        let rust_name = m.get_name().rust_name;
        let rust_ident = format_ident!("{}", rust_name);
        if !skip_method(m, rust_name) {
            Some(quote! { #rust_ident: 0 as *mut sys::godot_method_bind })
        } else {
            None
        }
    });

    let init_methods = class.methods.iter().filter_map(|m| {
        let MethodName {
            rust_name,
            original_name,
        } = m.get_name();

        let rust_ident = format_ident!("{}", rust_name);
        let original_name = format!("{original_name}\0");

        if !skip_method(m, rust_name) {
            assert!(original_name.ends_with('\0'), "original_name must be null terminated");
            Some(quote! {
                table.#rust_ident = (gd_api.godot_method_bind_get_method)(class_name, #original_name.as_ptr() as *const c_char );
            })
        } else {
            None
        }
    });

    assert!(
        lookup_name.ends_with('\0'),
        "lookup_name must be null terminated"
    );
    let methods = quote! {
        impl #method_table {
            unsafe fn get_mut() -> &'static mut Self {
                static mut TABLE: #method_table = #method_table {
                    class_constructor: None,
                    #(#impl_methods),*
                };

                &mut TABLE
            }

            #[inline]
            #[allow(dead_code)]
            pub fn get(gd_api: &GodotApi) -> &'static Self {
                unsafe {
                    let table = Self::get_mut();
                    static INIT: std::sync::Once = std::sync::Once::new();
                    INIT.call_once(|| {
                        #method_table::init(table, gd_api);
                    });

                    table
                }
            }

            #[inline(never)]
            #[allow(dead_code)]
            fn init(table: &mut Self, gd_api: &GodotApi) {
                unsafe {
                    let class_name = #lookup_name.as_ptr() as *const c_char;
                    table.class_constructor = (gd_api.godot_get_class_constructor)(class_name);
                    #(#init_methods)*
                }
            }
        }
    };

    quote! {
        #struct_definition
        #methods
    }
}

/// Removes 'get_' from the beginning of `name` if `name` is a property getter on `class`.
fn rename_property_getter<'a>(name: &'a str, class: &GodotClass) -> &'a str {
    if name.starts_with("get_") && class.is_getter(name) {
        &name[4..]
    } else {
        name
    }
}

const UNSAFE_OBJECT_METHODS: &[(&str, &str)] = &[
    ("Object", "call"),
    ("Object", "callv"),
    ("Object", "call_deferred"),
];

pub(crate) fn generate_methods(
    class: &GodotClass,
    icalls: &mut HashMap<String, MethodSig>,
    docs: Option<&GodotXmlDocs>,
) -> TokenStream {
    /// Memorized information about generated methods. Used to generate indexed property accessors.
    struct Generated {
        icall: proc_macro2::Ident,
        icall_ty: IcallType,
        maybe_unsafe: TokenStream,
        maybe_unsafe_reason: &'static str,
    }

    // Brings values of some types to a type with less information.
    fn arg_erase(ty: &Ty, name: &proc_macro2::Ident) -> TokenStream {
        match ty {
            Ty::VariantType | Ty::VariantOperator | Ty::Vector3Axis | Ty::Result => {
                quote! { (#name as u32) as i64 }
            }

            Ty::Variant => quote! { #name.owned_to_variant() },

            Ty::String | Ty::NodePath => quote! { #name.into() },

            Ty::Enum(_) => quote! { #name.0 },

            Ty::Object(_) => quote! { #name.as_arg_ptr() },

            // Allow lossy casting of numeric types into similar primitives, see also to_return_post
            Ty::I64 | Ty::F64 | Ty::Bool => quote! { #name as _ },

            _ => quote! { #name },
        }
    }

    let mut generated = HashMap::new();
    let mut result = TokenStream::new();

    for method in &class.methods {
        let MethodName {
            rust_name: method_name,
            ..
        } = method.get_name();

        if skip_method(method, method_name) {
            continue;
        }

        let mut ret_type = method.get_return_type();
        let mut rust_ret_type = ret_type.to_rust();

        // Ensure that methods are not injected several times.
        if generated.contains_key(method_name) {
            continue;
        }

        let mut params_decl = TokenStream::new();
        let mut params_use = TokenStream::new();
        for argument in &method.arguments {
            let ty = argument.get_type();
            let rust_ty = ty.to_rust_arg();
            let name = rust_safe_name(&argument.name);

            let arg_erased = arg_erase(&ty, &name);

            params_decl.extend(quote! {
                , #name: #rust_ty
            });
            params_use.extend(quote! {
                , #arg_erased
            });
        }

        if method.has_varargs {
            params_decl.extend(quote! {
                , varargs: &[Variant]
            });
            params_use.extend(quote! {
                , varargs
            });
            ret_type = Ty::Variant;
            rust_ret_type = syn::parse_quote! { Variant };
        }

        // Adjust getters to match guideline conventions:
        // https://rust-lang.github.io/api-guidelines/naming.html#getter-names-follow-rust-convention-c-getter
        let rusty_method_name = rename_property_getter(method_name, class);

        let method_sig = MethodSig::from_method(method);
        let icall_ty = method_sig.icall_type();
        let icall_name = method_sig.function_name();
        let icall = format_ident!("{}", icall_name);

        let maybe_unsafe: TokenStream;
        let maybe_unsafe_reason: &str;
        if let Some(unsafe_reason) = unsafe_reason(class, method_name, &method_sig) {
            maybe_unsafe = quote! { unsafe };
            maybe_unsafe_reason = unsafe_reason;
        } else {
            maybe_unsafe = TokenStream::default();
            maybe_unsafe_reason = "";
        }

        icalls.insert(icall_name.clone(), method_sig);

        let rusty_name = rust_safe_name(rusty_method_name);

        let method_bind_fetch = {
            let method_table = format_ident!("{}MethodTable", class.name);
            let rust_method_name = format_ident!("{}", method_name);

            quote! {
                let method_bind: *mut sys::godot_method_bind = #method_table::get(get_api()).#rust_method_name;
            }
        };

        let doc_comment = docs
            .and_then(|docs| docs.get_class_method_desc(class.name.as_str(), method_name))
            .unwrap_or("");

        let recover = ret_recover(&ret_type, icall_ty);

        let output = quote! {
            #[doc = #doc_comment]
            #[doc = #maybe_unsafe_reason]
            #[inline]
            pub #maybe_unsafe fn #rusty_name(&self #params_decl) -> #rust_ret_type {
                unsafe {
                    #method_bind_fetch

                    let ret = crate::icalls::#icall(method_bind, self.this.sys().as_ptr() #params_use);

                    #recover
                }
            }
        };

        result.extend(output);

        generated.insert(
            method_name.to_string(),
            Generated {
                icall,
                icall_ty,
                maybe_unsafe,
                maybe_unsafe_reason,
            },
        );
    }

    for property in &class.properties {
        if property.index < 0 || property.name.contains('/') {
            continue;
        }

        let property_index = property.index;
        let ty = Ty::from_src(&property.type_);

        if let Some(Generated {
            icall,
            icall_ty,
            maybe_unsafe,
            maybe_unsafe_reason,
        }) = generated.get(&property.getter)
        {
            let rusty_name = rust_safe_name(&property.name);
            let rust_ret_type = ty.to_rust();

            let method_bind_fetch = {
                let method_table = format_ident!("{}MethodTable", class.name);
                let rust_method_name = format_ident!("{}", property.getter);

                quote! {
                    let method_bind: *mut sys::godot_method_bind = #method_table::get(get_api()).#rust_method_name;
                }
            };

            let doc_comment = docs
                .and_then(|docs| {
                    docs.get_class_method_desc(
                        class.name.as_str(),
                        &format!("get_{}", property.name),
                    )
                })
                .unwrap_or("");

            let recover = ret_recover(&ty, *icall_ty);

            let output = quote! {
                #[doc = #doc_comment]
                #[doc = #maybe_unsafe_reason]
                #[inline]
                pub #maybe_unsafe fn #rusty_name(&self) -> #rust_ret_type {
                    unsafe {
                        #method_bind_fetch

                        let ret = crate::icalls::#icall(method_bind, self.this.sys().as_ptr(), #property_index);

                        #recover
                    }
                }
            };

            result.extend(output);
        }

        if let Some(Generated {
            icall,
            icall_ty,
            maybe_unsafe,
            maybe_unsafe_reason,
        }) = generated.get(&property.setter)
        {
            let rusty_name = rust_safe_name(&format!("set_{}", property.name));

            let rust_arg_ty = ty.to_rust_arg();
            let arg_ident = format_ident!("value");
            let arg_erased = arg_erase(&ty, &arg_ident);

            let method_bind_fetch = {
                let method_table = format_ident!("{}MethodTable", class.name);
                let rust_method_name = format_ident!("{}", property.setter);

                quote! {
                    let method_bind: *mut sys::godot_method_bind = #method_table::get(get_api()).#rust_method_name;
                }
            };

            let doc_comment = docs
                .and_then(|docs| {
                    docs.get_class_method_desc(
                        class.name.as_str(),
                        &format!("set_{}", property.name),
                    )
                })
                .unwrap_or("");

            let recover = ret_recover(&Ty::Void, *icall_ty);

            let output = quote! {
                #[doc = #doc_comment]
                #[doc = #maybe_unsafe_reason]
                #[inline]
                pub #maybe_unsafe fn #rusty_name(&self, #arg_ident: #rust_arg_ty) {
                    unsafe {
                        #method_bind_fetch

                        let ret = crate::icalls::#icall(method_bind, self.this.sys().as_ptr(), #property_index, #arg_erased);

                        #recover
                    }
                }
            };

            result.extend(output);
        }
    }

    result
}

/// Returns a message as to why this method would be unsafe; or None if the method is safe
fn unsafe_reason(
    class: &GodotClass,
    method_name: &str,
    method_sig: &MethodSig,
) -> Option<&'static str> {
    if UNSAFE_OBJECT_METHODS.contains(&(&class.name, method_name)) {
        Some(
            "\n# Safety\
             \nThis function bypasses Rust's static type checks (aliasing, thread boundaries, calls to free(), ...).",
        )
    } else if method_sig.arguments.contains(&Ty::Rid) {
        Some(
            "\n# Safety\
             \nThis function has parameters of type `Rid` (resource ID). \
             RIDs are untyped and interpreted as raw pointers by the engine, so passing an incorrect RID can cause UB.")
    } else {
        None
    }
}

fn ret_recover(ty: &Ty, icall_ty: IcallType) -> TokenStream {
    match icall_ty {
        #[cfg(feature = "ptrcall")]
        IcallType::Ptr => ty.to_return_post(),
        IcallType::Varargs => {
            // only variant possible
            quote! { ret }
        }
        IcallType::Var => ty.to_return_post_variant(),
    }
}

pub(crate) fn generate_icall(name: String, sig: MethodSig) -> TokenStream {
    match sig.icall_type() {
        #[cfg(feature = "ptrcall")]
        IcallType::Ptr => ptrcall::generate_icall(name, sig),
        IcallType::Varargs => varargs_call::generate_icall(name, sig),
        IcallType::Var => varcall::generate_icall(name, sig),
    }
}

#[cfg(feature = "ptrcall")]
mod ptrcall {
    use super::*;
    use quote::{format_ident, quote};

    pub(super) fn generate_icall(name: String, sig: super::MethodSig) -> proc_macro2::TokenStream {
        let name_ident = format_ident!("{}", name);

        let rust_ret_type = sig.return_type.to_icall_return();

        let arguments = sig
            .arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| (format_ident!("arg{}", i), ty));

        let args = arguments.clone().map(|(name, ty)| {
            let typ = ty.to_icall_arg();
            quote! {
                #name: #typ
            }
        });

        let arg_count = sig.arguments.len();
        let method_body = {
            let args = arguments
                .clone()
                .map(|(name, ty)| generate_argument_pre(ty, name));
            let return_pre = generate_return_pre(&sig.return_type);

            let arg_forgets = arguments.clone().map(|(name, _)| {
                quote! { let #name = ::std::mem::ManuallyDrop::new(#name); }
            });

            let arg_drops = arguments.clone().map(|(name, _)| {
                quote! { ::std::mem::ManuallyDrop::into_inner(#name); }
            });

            quote! {
                let gd_api = get_api();

                let mut argument_buffer : [*const libc::c_void; #arg_count] = [
                    #(#args),*
                ];

                #(#arg_forgets)*

                #return_pre

                (gd_api.godot_method_bind_ptrcall)(method_bind, obj_ptr, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);

                #(#arg_drops)*

                ret
            }
        };

        quote! {
            #[doc(hidden)]
            #[inline(never)]
            pub(crate) unsafe fn #name_ident(method_bind: *mut sys::godot_method_bind, obj_ptr: *mut sys::godot_object, #(#args,)*) -> #rust_ret_type {
                #method_body
            }
        }
    }

    fn generate_argument_pre(ty: &Ty, name: proc_macro2::Ident) -> TokenStream {
        match ty {
            Ty::Bool
            | Ty::F64
            | Ty::I64
            | Ty::Vector2
            | Ty::Vector3
            | Ty::Transform
            | Ty::Transform2D
            | Ty::Quat
            | Ty::Plane
            | Ty::Aabb
            | Ty::Basis
            | Ty::Rect2
            | Ty::Color => {
                quote! {
                    (&#name) as *const _ as *const _
                }
            }
            Ty::Variant
            | Ty::String
            | Ty::Rid
            | Ty::NodePath
            | Ty::VariantArray
            | Ty::Dictionary
            | Ty::ByteArray
            | Ty::StringArray
            | Ty::Vector2Array
            | Ty::Vector3Array
            | Ty::ColorArray
            | Ty::Int32Array
            | Ty::Float32Array => {
                quote! {
                    #name.sys() as *const _ as *const _
                }
            }
            Ty::Object(_) => {
                quote! {
                    #name as *const _ as *const _
                }
            }
            _ => Default::default(),
        }
    }

    fn generate_return_pre(ty: &Ty) -> TokenStream {
        match ty {
            Ty::Void => {
                quote! {
                    let ret = ();
                    let ret_ptr = ptr::null_mut();
                }
            }
            Ty::F64 => {
                quote! {
                    let mut ret = 0.0f64;
                    let ret_ptr = &mut ret as *mut _;
                }
            }
            Ty::I64 => {
                quote! {
                    let mut ret = 0i64;
                    let ret_ptr = &mut ret as *mut _;
                }
            }
            Ty::Bool => {
                quote! {
                    let mut ret = false;
                    let ret_ptr = &mut ret as *mut _;
                }
            }
            Ty::String
            | Ty::Vector2
            | Ty::Vector3
            | Ty::Transform
            | Ty::Transform2D
            | Ty::Quat
            | Ty::Plane
            | Ty::Rect2
            | Ty::Basis
            | Ty::Color
            | Ty::NodePath
            | Ty::Variant
            | Ty::Aabb
            | Ty::VariantArray
            | Ty::Dictionary
            | Ty::ByteArray
            | Ty::StringArray
            | Ty::Vector2Array
            | Ty::Vector3Array
            | Ty::ColorArray
            | Ty::Int32Array
            | Ty::Float32Array
            | Ty::Rid => {
                // Enum || Void is not handled here, can .unwrap()
                let sys_ty = ty.to_sys().unwrap();
                quote! {
                    let mut ret = #sys_ty::default();
                    let ret_ptr = (&mut ret) as *mut _;
                }
            }
            Ty::Object(_) => {
                quote! {
                    let mut ret: *mut sys::godot_object = ptr::null_mut();
                    let ret_ptr = (&mut ret) as *mut _;
                }
            }
            Ty::Result | Ty::VariantType | Ty::VariantOperator | Ty::Vector3Axis | Ty::Enum(_) => {
                quote! {
                    let mut ret = 0i64;
                    let ret_ptr = (&mut ret) as *mut _;
                }
            }
        }
    }
}

mod varargs_call {
    use crate::api::*;
    use quote::{format_ident, quote};

    pub(super) fn generate_icall(name: String, sig: super::MethodSig) -> proc_macro2::TokenStream {
        let name_ident = format_ident!("{}", name);

        let arguments = sig
            .arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| (format_ident!("arg{}", i), ty));

        let args = arguments.clone().map(|(name, ty)| {
            let rust_ty = ty.to_icall_arg();
            quote! {
                #name: #rust_ty
            }
        });

        let arg_count = sig.arguments.len();
        let method_body = {
            let args_buffer = quote! {
                let mut argument_buffer: std::vec::Vec<*const sys::godot_variant> = std::vec::Vec::with_capacity(#arg_count + varargs.len());
            };

            let args = arguments.clone().map(|(name, ty)| {
                let new_var = match ty {
                    Ty::Object(_) => {
                        quote! {
                            let #name: Variant = Variant::from_object_ptr(#name);
                        }
                    }
                    _ => {
                        quote! {
                           let #name: Variant = (&#name).to_variant();
                        }
                    }
                };

                quote! {
                    #new_var
                    argument_buffer.push(#name.sys());
                }
            });

            let varargs_body = quote! {
                for arg in varargs {
                    argument_buffer.push(arg.sys() as *const _);
                }
                let ret = (gd_api.godot_method_bind_call)(method_bind, obj_ptr, argument_buffer.as_mut_ptr(), argument_buffer.len() as _, ptr::null_mut());
            };

            let drop_args = arguments.clone().map(|(name, _)| {
                quote! { drop(#name); }
            });

            quote! {
                let gd_api = get_api();

                #args_buffer
                #(#args)*
                #varargs_body
                #(#drop_args)*
                Variant::from_sys(ret)
            }
        };

        quote! {
            #[doc(hidden)]
            #[inline(never)]
            pub(crate) unsafe fn #name_ident(method_bind: *mut sys::godot_method_bind, obj_ptr: *mut sys::godot_object, #(#args,)* varargs: &[Variant]) -> Variant {
                #method_body
            }
        }
    }
}

mod varcall {
    use crate::api::*;
    use quote::{format_ident, quote};

    pub(super) fn generate_icall(name: String, sig: super::MethodSig) -> proc_macro2::TokenStream {
        let name_ident = format_ident!("{}", name);

        let arguments = sig
            .arguments
            .iter()
            .enumerate()
            .map(|(i, ty)| (format_ident!("arg{}", i), ty));

        let args = arguments.clone().map(|(name, ty)| {
            let rust_ty = ty.to_icall_arg();
            quote! {
                #name: #rust_ty
            }
        });

        let arg_count = sig.arguments.len();
        let method_body = {
            let arg_bindings = arguments.clone().map(|(name, ty)| match ty {
                Ty::Object(_) => {
                    quote! {
                        let #name: Variant = Variant::from_object_ptr(#name);
                    }
                }
                _ => {
                    quote! {
                       let #name: Variant = (&#name).to_variant();
                    }
                }
            });

            let args_buffer = {
                let arg_assigns = arguments.clone().map(|(name, _)| quote! {#name});
                quote! {

                    let mut argument_buffer: [*const sys::godot_variant; #arg_count] = [
                        #(#arg_assigns.sys() as *const _),*
                    ];
                }
            };

            let body = quote! {
                let ret = (gd_api.godot_method_bind_call)(method_bind, obj_ptr, argument_buffer.as_mut_ptr(), argument_buffer.len() as _, ptr::null_mut());
            };

            let drop_args = arguments.clone().map(|(name, _)| {
                quote! { drop(#name); }
            });

            quote! {
                let gd_api = get_api();

                #(#arg_bindings)*
                #args_buffer
                #body
                #(#drop_args)*

                Variant::from_sys(ret)
            }
        };

        quote! {
            #[doc(hidden)]
            #[inline(never)]
            pub(crate) unsafe fn #name_ident(method_bind: *mut sys::godot_method_bind, obj_ptr: *mut sys::godot_object, #(#args,)*) -> Variant {
                #method_body
            }
        }
    }
}

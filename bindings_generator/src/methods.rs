use crate::api::*;
use crate::documentation::class_doc_link;
use crate::rust_safe_name;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::collections::HashSet;

fn skip_method(name: &str) -> bool {
    const METHODS: &[&str] = &["free", "reference", "unreference"];
    METHODS.contains(&name)
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
        let rust_name = format_ident!("{}", m.get_name().rust_name);
        if rust_name != "free" {
            Some(quote! { pub #rust_name: *mut sys::godot_method_bind })
        } else {
            None
        }
    });

    let struct_definition = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub(crate) struct #method_table {
            pub class_constructor: sys::godot_class_constructor,
            #(#struct_methods),*
        }
    };

    let impl_methods = class.methods.iter().filter_map(|m| {
        let rust_name = format_ident!("{}", m.get_name().rust_name);
        if rust_name != "free" {
            Some(quote! { #rust_name: 0 as *mut sys::godot_method_bind })
        } else {
            None
        }
    });

    let init_methods = class.methods.iter().filter_map(|m| {
        let MethodName {
            rust_name,
            original_name,
        } = m.get_name();

        let rust_name = format_ident!("{}", rust_name);
        let original_name = format!("{}\0", original_name);

        if rust_name != "free" {
            assert!(original_name.ends_with('\0'), "original_name must be null terminated");
            Some(quote! {
                table.#rust_name = (gd_api.godot_method_bind_get_method)(class_name, #original_name.as_ptr() as *const c_char );
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
            pub fn get(gd_api: &GodotApi) -> &'static Self {
                unsafe {
                    let table = Self::get_mut();
                    static INIT: ::std::sync::Once = Once::new();
                    INIT.call_once(|| {
                        #method_table::init(table, gd_api);
                    });

                    table
                }
            }

            #[inline(never)]
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

pub fn generate_method_impl(class: &GodotClass, method: &GodotMethod) -> TokenStream {
    let MethodName {
        rust_name: method_name,
        ..
    } = method.get_name();

    if skip_method(&method_name) {
        return Default::default();
    }

    let rust_ret_type = if method.has_varargs {
        Ty::Variant.to_rust()
    } else {
        method.get_return_type().to_rust()
    };

    let args = method.arguments.iter().map(|argument| {
        let name = format_ident!("{}", rust_safe_name(&argument.name));
        let typ = argument.get_type().to_rust_arg();
        quote! {
            #name: #typ
        }
    });

    let varargs = if method.has_varargs {
        quote! {
            varargs: &[Variant]
        }
    } else {
        Default::default()
    };

    let arg_count = method.arguments.len();
    let method_body = if method.has_varargs {
        let args_buffer = quote! {
            let mut argument_buffer: Vec<*const sys::godot_variant> = Vec::with_capacity(#arg_count + varargs.len());
        };

        let arguments = method.arguments.iter().map(|arg| {
            let name = rust_safe_name(&arg.name);

            let new_var = match arg.get_type() {
                Ty::Object(_) => {
                    quote! {
                        let #name: Variant = #name.to_arg_variant();
                    }
                }
                Ty::String => {
                    quote! {
                        let #name: Variant = Variant::from_godot_string(&#name);
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
            let ret = Variant::from_sys((gd_api.godot_method_bind_call)(method_bind, obj_ptr, argument_buffer.as_mut_ptr(), argument_buffer.len() as _, ptr::null_mut()));
        };

        let drop_args = method.arguments.iter().map(|arg| {
            let arg_name = rust_safe_name(&arg.name);
            quote! { drop(#arg_name); }
        });

        let ret = match method.get_return_type() {
            Ty::Object(_) => quote! { ret.try_to_object() },
            _ => quote! { ret.into() },
        };

        quote! {
            #args_buffer
            #(#arguments)*
            #varargs_body
            #(#drop_args)*
            #ret
        }
    } else {
        let args = method
            .arguments
            .iter()
            .map(|arg| generate_argument_pre(&arg.get_type(), rust_safe_name(&arg.name)));
        let return_pre = generate_return_pre(&method.get_return_type());

        let arg_drops = method.arguments.iter().map(|arg| {
            let name = rust_safe_name(&arg.name);
            quote! { drop(#name); }
        });

        let ret = method.get_return_type().to_return_post();

        quote! {
            let mut argument_buffer : [*const libc::c_void; #arg_count] = [
                #(#args),*
            ];

            #return_pre

            (gd_api.godot_method_bind_ptrcall)(method_bind, obj_ptr, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);

            #(#arg_drops)*

            #ret
        }
    };

    let class_method_name = format_ident!("{}_{}", class.name, method_name);
    let method_table = format_ident!("{}MethodTable", class.name);
    let rust_method_name = format_ident!("{}", method_name);
    let visibility = if class.name == "Reference" && method_name == "init_ref" {
        quote! { pub }
    } else {
        quote! { pub(crate) }
    };

    quote! {
        #[doc(hidden)]
        #[inline]
        #visibility unsafe fn #class_method_name(obj_ptr: *mut sys::godot_object, #(#args,)* #varargs) -> #rust_ret_type {
            let gd_api = get_api();

            let method_bind: *mut sys::godot_method_bind = #method_table::get(gd_api).#rust_method_name;
            #method_body
        }
    }
}

/// Removes 'get_' from the beginning of `name` if `name` is a property getter on `class`.
fn rename_property_getter<'a>(name: &'a str, class: &GodotClass) -> &'a str {
    if name.starts_with("get_") && class.is_getter(name) {
        &name[4..]
    } else {
        &name[..]
    }
}

const UNSAFE_OBJECT_METHODS: &[(&'static str, &'static str)] = &[
    ("Object", "call"),
    ("Object", "vcall"),
    ("Object", "call_deferred"),
];

pub fn generate_methods(
    api: &Api,
    method_set: &mut HashSet<String>,
    class_name: &str,
    is_safe: bool,
    is_leaf: bool,
) -> TokenStream {
    let mut result = TokenStream::new();
    if let Some(class) = api.find_class(class_name) {
        for method in &class.methods {
            let MethodName {
                rust_name: method_name,
                ..
            } = method.get_name();

            if skip_method(&method_name) {
                continue;
            }

            let mut rust_ret_type = method.get_return_type().to_rust();

            // Ensure that methods are not injected several times.
            let method_name_string = method_name.to_string();
            if method_set.contains(&method_name_string) {
                continue;
            }
            method_set.insert(method_name_string);

            let mut params_decl = TokenStream::new();
            let mut params_use = TokenStream::new();
            for argument in &method.arguments {
                let ty = argument.get_type().to_rust_arg();
                let name = rust_safe_name(&argument.name);
                params_decl.extend(quote! {
                    , #name: #ty
                });
                params_use.extend(quote! {
                    , #name
                });
            }

            if method.has_varargs {
                params_decl.extend(quote! {
                    , varargs: &[Variant]
                });
                params_use.extend(quote! {
                    , varargs
                });
                rust_ret_type = syn::parse_quote! { Variant };
            }

            if !is_leaf {
                let documentation = format!("    /// Inherited from {}.", class_doc_link(class));
                result.extend(quote! {
                    #[doc=#documentation]
                });
            }

            // Adjust getters to match guideline conventions:
            // https://rust-lang.github.io/api-guidelines/naming.html#getter-names-follow-rust-convention-c-getter
            let rusty_method_name = rename_property_getter(&method_name, &class);

            let rusty_name = format_ident!("{}", rusty_method_name);
            let function_name = format_ident!("{}_{}", class.name, method_name);

            let maybe_unsafe = if UNSAFE_OBJECT_METHODS.contains(&(&class.name, method_name)) {
                quote! { unsafe }
            } else {
                Default::default()
            };

            let output = quote! {
                #[inline]
                pub #maybe_unsafe fn #rusty_name(&self #params_decl) -> #rust_ret_type {
                    unsafe { #function_name(self.this.sys().as_ptr() #params_use) }
                }
            };
            result.extend(output);
        }

        // Reference includes all of Object's methods so they are safe.
        if class.base_class == "Reference" {
            result.extend(generate_methods(
                api,
                method_set,
                &class.base_class,
                is_safe,
                false,
            ));
        }
    }
    result
}

fn generate_argument_pre(ty: &Ty, name: proc_macro2::Ident) -> TokenStream {
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
            quote! {
                (&#name) as *const _ as *const _
            }
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
            quote! {
                #name.sys() as *const _ as *const _
            }
        }
        &Ty::Object(_) => {
            quote! {
                #name.as_arg_ptr() as *const _ as *const _
            }
        }
        _ => Default::default(),
    }
}

fn generate_return_pre(ty: &Ty) -> TokenStream {
    match ty {
        &Ty::Void => {
            quote! {
                let ret_ptr = ptr::null_mut();
            }
        },
        &Ty::F64 => {
            quote !{
                let mut ret = 0.0f64;
                let ret_ptr = &mut ret as *mut _;
            }
        },
        &Ty::I64 => {
            quote !{
                let mut ret = 0i64;
                let ret_ptr = &mut ret as *mut _;
            }
        },
        &Ty::Bool => {
            quote !{
                let mut ret = false;
                let ret_ptr = &mut ret as *mut _;
            }
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
            // Enum || Void is not handled here, can .unwrap()
            let sys_ty = ty.to_sys().unwrap();
            quote !{
                let mut ret = #sys_ty::default();
                let ret_ptr = &mut ret as *mut _;
            }
        }
        &Ty::Object(_) // TODO: double check
        => {
            quote!{
                let mut ret: *mut sys::godot_object = ptr::null_mut();
                let ret_ptr = (&mut ret) as *mut _;
            }
        }
        &Ty::Result => {
            quote! {
                let mut ret: sys::godot_error = sys::godot_error_GODOT_OK;
                let ret_ptr = (&mut ret) as *mut _;
            }
        }
        &Ty::VariantType => {
            quote! {
                let mut ret: sys::godot_variant_type = sys::godot_variant_type_GODOT_VARIANT_TYPE_NIL;
                let ret_ptr = (&mut ret) as *mut _;
            }
        }
        &Ty::VariantOperator => {
            // An invalid value is used here, so that `try_from_sys` can detect the error in case
            // the pointer is not written to.
            quote! {
                let mut ret: sys::godot_variant_operator = sys::godot_variant_operator_GODOT_VARIANT_OP_MAX;
                let ret_ptr = (&mut ret) as *mut _;
            }
        }
        &Ty::Enum(ref name) => {
            let name = format_ident!("{}", name);
            quote! {
                let mut ret: #name = mem::transmute(0);
                let ret_ptr = (&mut ret) as *mut _;
            }
        }
    }
}

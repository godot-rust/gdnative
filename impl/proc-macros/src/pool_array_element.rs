use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Type};

#[derive(Debug)]
struct Args {
    trait_name: Ident,
    rust_ty: Type,
    sys_ty: Option<Type>,
    sys_ref_ty: Option<Type>,
    godot_type: String,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        input.parse::<Token![impl]>()?;

        let trait_name = input.parse()?;

        input.parse::<Token![for]>()?;

        let rust_ty = input.parse()?;

        let sys_ty = input
            .parse::<Token![as]>()
            .ok()
            .map(|_| input.parse())
            .transpose()?;

        let sys_ref_ty = input
            .parse::<Token![ref]>()
            .ok()
            .map(|_| input.parse())
            .transpose()?;

        input.parse::<Token![=>]>()?;

        let godot_type = input.parse::<Ident>()?;
        let godot_type = godot_type.to_string();

        let brace_content;
        braced!(brace_content in input);

        brace_content.parse::<Token![..]>()?;

        if !brace_content.is_empty() {
            return Err(brace_content.error("expecting only `..` in braces"));
        }

        if !input.is_empty() {
            return Err(input.error("expecting end of macro input"));
        }

        Ok(Args {
            trait_name,
            rust_ty,
            sys_ty,
            sys_ref_ty,
            godot_type,
        })
    }
}

static METHODS: &[&str] = &[
    "new",
    "new_copy",
    "new_with_array",
    "append",
    "append_array",
    "insert",
    "invert",
    "push_back",
    "remove",
    "resize",
    "read",
    "write",
    "set",
    "get",
    "size",
    "destroy",
    "read_access_copy",
    "read_access_ptr",
    "read_access_operator_assign",
    "read_access_destroy",
    "write_access_copy",
    "write_access_ptr",
    "write_access_operator_assign",
    "write_access_destroy",
];

fn impl_fn_symbol(method: &str) -> Ident {
    Ident::new(&format!("{method}_fn"), Span::call_site())
}

fn fn_symbol(godot_type: &str, method: &str) -> Ident {
    Ident::new(
        &format!("godot_pool_{godot_type}_array_{method}"),
        Span::call_site(),
    )
}

fn fn_ty(method: &str) -> Type {
    match method {
        "new" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray)),
        "new_copy" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, *const Self::SysArray))
        }
        "new_with_array" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, *const sys::godot_array))
        }
        "append" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, Self::SysRefTy)),
        "append_array" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, *const Self::SysArray))
        }
        "insert" => parse_quote!(
            unsafe extern "C" fn(
                *mut Self::SysArray,
                sys::godot_int,
                Self::SysRefTy,
            ) -> sys::godot_error
        ),
        "invert" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray)),
        "push_back" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, Self::SysRefTy)),
        "remove" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, sys::godot_int)),
        "resize" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, sys::godot_int)),
        "read" => {
            parse_quote!(unsafe extern "C" fn(*const Self::SysArray) -> *mut Self::SysReadAccess)
        }
        "write" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysArray) -> *mut Self::SysWriteAccess)
        }
        "set" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysArray, sys::godot_int, Self::SysRefTy))
        }
        "get" => {
            parse_quote!(unsafe extern "C" fn(*const Self::SysArray, sys::godot_int) -> Self::SysTy)
        }
        "size" => parse_quote!(unsafe extern "C" fn(*const Self::SysArray) -> sys::godot_int),
        "destroy" => parse_quote!(unsafe extern "C" fn(*mut Self::SysArray)),
        "read_access_copy" => parse_quote!(
            unsafe extern "C" fn(*const Self::SysReadAccess) -> *mut Self::SysReadAccess
        ),
        "read_access_ptr" => {
            parse_quote!(unsafe extern "C" fn(*const Self::SysReadAccess) -> *const Self::SysTy)
        }
        "read_access_operator_assign" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysReadAccess, *mut Self::SysReadAccess))
        }
        "read_access_destroy" => parse_quote!(unsafe extern "C" fn(*mut Self::SysReadAccess)),
        "write_access_copy" => parse_quote!(
            unsafe extern "C" fn(*const Self::SysWriteAccess) -> *mut Self::SysWriteAccess
        ),
        "write_access_ptr" => {
            parse_quote!(unsafe extern "C" fn(*const Self::SysWriteAccess) -> *mut Self::SysTy)
        }
        "write_access_operator_assign" => {
            parse_quote!(unsafe extern "C" fn(*mut Self::SysWriteAccess, *mut Self::SysWriteAccess))
        }
        "write_access_destroy" => parse_quote!(unsafe extern "C" fn(*mut Self::SysWriteAccess)),
        _ => panic!("unknown method: {method}"),
    }
}

#[allow(clippy::unnecessary_wraps)]
fn expand(input: Args) -> Result<TokenStream, syn::Error> {
    let Args {
        trait_name,
        rust_ty,
        sys_ty,
        sys_ref_ty,
        godot_type,
    } = input;

    let (to_sys_fn, from_sys_fn) = if sys_ty.is_none() {
        let to_sys_fn = quote! {
            #[inline(always)]
            fn element_to_sys(self) -> Self::SysTy {
                self
            }
        };

        let from_sys_fn = quote! {
            #[inline(always)]
            fn element_from_sys(sys: Self::SysTy) -> Self {
                sys
            }
        };

        (to_sys_fn, from_sys_fn)
    } else {
        let to_sys_fn = quote! {
            #[inline(always)]
            fn element_to_sys(self) -> Self::SysTy {
                self.to_sys()
            }
        };

        let from_sys_fn = quote! {
            #[inline(always)]
            fn element_from_sys(sys: Self::SysTy) -> Self {
                Self::from_sys(sys)
            }
        };

        (to_sys_fn, from_sys_fn)
    };

    let sys_ty = sys_ty.unwrap_or_else(|| rust_ty.clone());

    let (to_sys_ref_fn, from_sys_ref_fn) = if sys_ref_ty.is_none() {
        let to_sys_ref_fn = quote! {
            #[inline(always)]
            fn element_to_sys_ref(&self) -> Self::SysRefTy {
                self.element_to_sys()
            }
        };

        let from_sys_ref_fn = quote! {
            #[inline(always)]
            unsafe fn element_from_sys_ref(sys: Self::SysRefTy) -> Self {
                Self::element_from_sys(sys)
            }
        };

        (to_sys_ref_fn, from_sys_ref_fn)
    } else {
        let to_sys_ref_fn = quote! {
            #[inline(always)]
            fn element_to_sys_ref(&self) -> Self::SysRefTy {
                self.sys()
            }
        };

        let from_sys_ref_fn = quote! {
            #[inline(always)]
            unsafe fn element_from_sys_ref(sys: Self::SysRefTy) -> Self {
                Self::from_sys(*sys)
            }
        };

        (to_sys_ref_fn, from_sys_ref_fn)
    };

    let sys_ref_ty = sys_ref_ty.unwrap_or_else(|| sys_ty.clone());

    let sys_array = Ident::new(
        &format!("godot_pool_{}_array", &godot_type),
        Span::call_site(),
    );
    let sys_array: Type = parse_quote!(sys::#sys_array);
    let sys_read_access = Ident::new(
        &format!("godot_pool_{}_array_read_access", &godot_type),
        Span::call_site(),
    );
    let sys_read_access: Type = parse_quote!(sys::#sys_read_access);
    let sys_write_access = Ident::new(
        &format!("godot_pool_{}_array_write_access", &godot_type),
        Span::call_site(),
    );
    let sys_write_access: Type = parse_quote!(sys::#sys_write_access);

    let array_to_variant_fn_symbol = Ident::new(
        &format!("godot_variant_new_pool_{}_array", &godot_type),
        Span::call_site(),
    );

    let array_from_variant_fn_symbol = Ident::new(
        &format!("godot_variant_as_pool_{}_array", &godot_type),
        Span::call_site(),
    );

    let sys_variant_type_symbol = Ident::new(
        &format!(
            "godot_variant_type_GODOT_VARIANT_TYPE_POOL_{}_ARRAY",
            godot_type.to_ascii_uppercase()
        ),
        Span::call_site(),
    );

    let functions = METHODS.iter().map(|method| {
        let fn_symbol = fn_symbol(&godot_type, method);
        let fn_ty = fn_ty(method);
        let impl_fn_symbol = impl_fn_symbol(method);

        quote! {
            #[inline(always)]
            fn #impl_fn_symbol (api: &sys::GodotApi) -> #fn_ty {
                api.#fn_symbol
            }
        }
    });

    Ok(quote! {
        impl private::Sealed for #rust_ty {}
        #[allow(unused_unsafe)]
        impl #trait_name for #rust_ty {
            type SysArray = #sys_array;
            type SysReadAccess = #sys_read_access;
            type SysWriteAccess = #sys_write_access;
            type SysTy = #sys_ty;
            type SysRefTy = #sys_ref_ty;

            const SYS_VARIANT_TYPE: sys::godot_variant_type = sys::#sys_variant_type_symbol;

            #to_sys_fn
            #from_sys_fn
            #to_sys_ref_fn
            #from_sys_ref_fn

            #[inline(always)]
            fn array_to_variant_fn(
                api: &sys::GodotApi,
            ) -> unsafe extern "C" fn (*mut sys::godot_variant, *const Self::SysArray) {
                api.#array_to_variant_fn_symbol
            }

            #[inline(always)]
            fn array_from_variant_fn(
                api: &sys::GodotApi,
            ) -> unsafe extern "C" fn (*const sys::godot_variant) -> Self::SysArray {
                api.#array_from_variant_fn_symbol
            }

            #(#functions)*
        }
    })
}

pub fn impl_element(input: proc_macro::TokenStream) -> Result<TokenStream, syn::Error> {
    syn::parse(input).and_then(expand)
}

struct DeclArgs {
    attrs: Vec<syn::Attribute>,
    trait_name: Ident,
}

impl Parse for DeclArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        input.parse::<Token![pub]>()?;
        input.parse::<Token![trait]>()?;

        let trait_name = input.parse()?;

        input.parse::<Token![:]>()?;

        if (input.parse::<Ident>()?) != "private" {
            return Err(input.error("expecting a `private::Sealed` supertrait"));
        }

        input.parse::<Token![::]>()?;

        if (input.parse::<Ident>()?) != "Sealed" {
            return Err(input.error("expecting a `private::Sealed` supertrait"));
        }

        let brace_content;
        braced!(brace_content in input);

        brace_content.parse::<Token![..]>()?;

        if !brace_content.is_empty() {
            return Err(brace_content.error("expecting only `..` in braces"));
        }

        if !input.is_empty() {
            return Err(input.error("expecting end of macro input"));
        }

        Ok(DeclArgs { attrs, trait_name })
    }
}

pub fn decl_element(input: proc_macro::TokenStream) -> Result<TokenStream, syn::Error> {
    let DeclArgs { attrs, trait_name } = syn::parse(input)?;

    let functions = METHODS.iter().map(|method| {
        let fn_ty = fn_ty(method);
        let impl_fn_symbol = impl_fn_symbol(method);

        quote! {
            #[doc(hidden)]
            fn #impl_fn_symbol (api: &sys::GodotApi) -> #fn_ty;
        }
    });

    Ok(quote! {
        #(#attrs)*
        pub trait #trait_name: private::Sealed {
            #[doc(hidden)]
            type SysArray: std::default::Default;
            #[doc(hidden)]
            type SysReadAccess;
            #[doc(hidden)]
            type SysWriteAccess;
            #[doc(hidden)]
            type SysTy: std::default::Default;
            #[doc(hidden)]
            type SysRefTy;

            #[doc(hidden)]
            const SYS_VARIANT_TYPE: sys::godot_variant_type;

            #[doc(hidden)]
            fn element_to_sys(self) -> Self::SysTy;

            #[doc(hidden)]
            fn element_to_sys_ref(&self) -> Self::SysRefTy;

            #[doc(hidden)]
            fn element_from_sys(sys: Self::SysTy) -> Self;

            #[doc(hidden)]
            unsafe fn element_from_sys_ref(sys: Self::SysRefTy) -> Self;

            #[doc(hidden)]
            fn array_to_variant_fn(
                api: &sys::GodotApi,
            ) -> unsafe extern "C" fn (*mut sys::godot_variant, *const Self::SysArray);

            #[doc(hidden)]
            fn array_from_variant_fn(
                api: &sys::GodotApi,
            ) -> unsafe extern "C" fn (*const sys::godot_variant) -> Self::SysArray;

            #(#functions)*
        }
    })
}

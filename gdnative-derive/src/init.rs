use proc_macro2::{Ident, Span, TokenStream};
use syn::{spanned::Spanned, AttributeArgs, ItemImpl, Lit, Meta, NestedMeta};

pub(crate) fn derive_callbacks(
    args: AttributeArgs,
    item_impl: ItemImpl,
) -> Result<TokenStream, syn::Error> {
    let mut prefix = None;
    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(meta)) if meta.path.is_ident("prefix") => {
                let span = meta.span();
                if prefix.replace(meta.lit).is_some() {
                    return Err(syn::Error::new(span, "duplicate argument"));
                }
            }
            NestedMeta::Meta(meta) if meta.path().is_ident("prefix") => {
                return Err(syn::Error::new(meta.span(), "expecting a name-value pair"));
            }
            _ => {
                return Err(syn::Error::new(arg.span(), "unknown argument"));
            }
        }
    }

    let prefix = match prefix {
        Some(Lit::Str(s)) => s.value(),
        Some(lit) => return Err(syn::Error::new(lit.span(), "expecting string literal")),
        None => "godot_".into(),
    };

    let derived = crate::automatically_derived();
    let gdnative_core = crate::crate_gdnative_core();
    let self_ty = &item_impl.self_ty;

    if !item_impl.generics.params.is_empty() {
        return Err(syn::Error::new(self_ty.span(), "generics are unsupported"));
    }

    let gdnative_init = Ident::new(&format!("{prefix}gdnative_init"), Span::call_site());
    let gdnative_terminate = Ident::new(&format!("{prefix}gdnative_terminate"), Span::call_site());
    let gdnative_singleton = Ident::new(&format!("{prefix}gdnative_singleton"), Span::call_site());
    let nativescript_init = Ident::new(&format!("{prefix}nativescript_init"), Span::call_site());
    let nativescript_terminate = Ident::new(
        &format!("{prefix}nativescript_terminate"),
        Span::call_site(),
    );
    let nativescript_frame = Ident::new(&format!("{prefix}nativescript_frame"), Span::call_site());
    let nativescript_thread_enter = Ident::new(
        &format!("{prefix}nativescript_thread_enter"),
        Span::call_site(),
    );
    let nativescript_thread_exit = Ident::new(
        &format!("{prefix}nativescript_thread_exit"),
        Span::call_site(),
    );

    Ok(quote! {
        #item_impl

        #derived
        const _: () = {
            impl #gdnative_core::init::private::TheGDNativeCallbacksAttributeIsRequired for #self_ty {}

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #gdnative_init(
                options: *mut #gdnative_core::sys::godot_gdnative_init_options,
            ) {
                #gdnative_core::init::private::gdnative_init::<#self_ty>(options);
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #gdnative_terminate(
                options: *mut #gdnative_core::sys::godot_gdnative_terminate_options,
            ) {
                #gdnative_core::init::private::gdnative_terminate::<#self_ty>(options);
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #gdnative_singleton() {
                #gdnative_core::init::private::gdnative_singleton::<#self_ty>();
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #nativescript_init(
                handle: *mut #gdnative_core::libc::c_void,
            ) {
                #gdnative_core::init::private::nativescript_init::<#self_ty>(handle);
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #nativescript_terminate(
                handle: *mut #gdnative_core::libc::c_void,
            ) {
                #gdnative_core::init::private::nativescript_terminate::<#self_ty>(handle);
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #nativescript_frame() {
                #gdnative_core::init::private::nativescript_frame::<#self_ty>();
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #nativescript_thread_enter() {
                #gdnative_core::init::private::nativescript_thread_enter::<#self_ty>();
            }

            #[no_mangle]
            #[doc(hidden)]
            #[allow(unused_unsafe)]
            pub unsafe extern "C" fn #nativescript_thread_exit() {
                #gdnative_core::init::private::nativescript_thread_exit::<#self_ty>();
            }
        };
    })
}

use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, Fields};

pub(crate) fn derive_export(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let derived_enum = match &input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[derive(Export)] can only use on enum",
            ))
        }
    };

    let export_impl = impl_export(&input.ident, derived_enum)?;
    Ok(export_impl)
}

fn impl_export(enum_ty: &syn::Ident, data: &syn::DataEnum) -> syn::Result<TokenStream2> {
    let mappings = {
        let mut m = Vec::with_capacity(data.variants.len());

        for variant in &data.variants {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(syn::Error::new(
                    variant.ident.span(),
                    "#[derive(Export)] only supports fieldless enums",
                ));
            }
            let key = &variant.ident;
            let val = quote! { #enum_ty::#key as i64 };
            m.push(quote! { (stringify!(#key).to_string(), #val) });
        }

        m
    };
    let impl_block = quote! {
        impl ::gdnative::export::Export for #enum_ty {
            type Hint = ::gdnative::export::hint::IntHint<i64>;
            #[inline]
            fn export_info(hint: Option<Self::Hint>) -> ::gdnative::export::ExportInfo {
                if let Some(hint) = hint {
                    return hint.export_info();
                } else {
                    let mappings = vec![ #(#mappings),* ];
                    let enum_hint = ::gdnative::export::hint::EnumHint::with_numbers(mappings);
                    return ::gdnative::export::hint::IntHint::<i64>::Enum(enum_hint).export_info();
                }
            }
        }
    };

    Ok(impl_block)
}

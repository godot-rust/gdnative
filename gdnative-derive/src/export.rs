use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::spanned::Spanned;
use syn::{DeriveInput, Fields};

fn err_only_supports_fieldless_enums(span: Span) -> syn::Error {
    syn::Error::new(span, "#[derive(Export)] only supports fieldless enums")
}

pub(crate) fn derive_export(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let derived_enum = match &input.data {
        syn::Data::Enum(data) => data,
        syn::Data::Struct(data) => {
            return Err(err_only_supports_fieldless_enums(data.struct_token.span()));
        }
        syn::Data::Union(data) => {
            return Err(err_only_supports_fieldless_enums(data.union_token.span()));
        }
    };

    let export_impl = impl_export(&input.ident, derived_enum)?;
    Ok(export_impl)
}

fn impl_export(enum_ty: &syn::Ident, data: &syn::DataEnum) -> syn::Result<TokenStream2> {
    let err = data
        .variants
        .iter()
        .filter(|variant| !matches!(variant.fields, Fields::Unit))
        .map(|variant| err_only_supports_fieldless_enums(variant.ident.span()))
        .reduce(|mut acc, err| {
            acc.combine(err);
            acc
        });
    if let Some(err) = err {
        return Err(err);
    }

    let mappings = data
        .variants
        .iter()
        .map(|variant| {
            let key = &variant.ident;
            let val = quote! { #enum_ty::#key as i64 };
            quote! { (stringify!(#key).to_string(), #val) }
        })
        .collect::<Vec<_>>();

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

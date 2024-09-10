use crate::crate_gdnative_core;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{DeriveInput, Fields, Meta};

#[derive(Copy, Clone, Debug)]
enum Kind {
    Enum,
}

#[derive(Debug)]
struct DeriveData {
    kind: Kind,
    ident: Ident,
    data: syn::Data,
}

fn parse_derive_input(input: DeriveInput) -> syn::Result<DeriveData> {
    let DeriveInput {
        ident, data, attrs, ..
    } = input.clone();

    let (kind, errors) = attrs
        .iter()
        .filter(|attr| attr.path.is_ident("export"))
        .fold((None, vec![]), |(mut kind, mut errors), attr| {
            let list = match attr.parse_meta() {
                Ok(meta) => match meta {
                    Meta::List(list) => list,
                    Meta::Path(path) => {
                        errors.push(syn::Error::new(
                            path.span(),
                            "missing macro arguments. expected #[export(...)]",
                        ));
                        return (kind, errors);
                    }
                    Meta::NameValue(pair) => {
                        errors.push(syn::Error::new(
                            pair.span(),
                            "missing macro arguments. expected #[export(...)]",
                        ));
                        return (kind, errors);
                    }
                },
                Err(e) => {
                    errors.push(syn::Error::new(
                        e.span(),
                        format!("unknown attribute format. expected #[export(...)]: {e}"),
                    ));
                    return (kind, errors);
                }
            };

            for meta in list.nested.into_iter() {
                let syn::NestedMeta::Meta(Meta::NameValue(pair)) = meta else {
                    errors.push(syn::Error::new(
                        meta.span(),
                        "invalid syntax. expected #[export(key = \"value\")]",
                    ));
                    continue;
                };

                if !pair.path.is_ident("kind") {
                    errors.push(syn::Error::new(
                        pair.span(),
                        format!("found {}, expected kind", pair.path.into_token_stream()),
                    ));
                    continue;
                }

                let syn::Lit::Str(str) = pair.lit else {
                    errors.push(syn::Error::new(
                        pair.lit.span(),
                        "string literal expected, wrap with double quotes",
                    ));
                    continue;
                };

                match str.value().as_str() {
                    "enum" => {
                        if kind.is_some() {
                            errors.push(syn::Error::new(str.span(), "kind already set"));
                        } else {
                            kind = Some(Kind::Enum);
                        }
                    }
                    _ => {
                        errors.push(syn::Error::new(str.span(), "unknown kind, expected enum"));
                    }
                }
            }

            (kind, errors)
        });

    if let Some(err) = errors.into_iter().reduce(|mut acc, err| {
        acc.combine(err);
        acc
    }) {
        return Err(err);
    }

    match kind {
        Some(kind) => Ok(DeriveData { ident, kind, data }),
        None => Err(syn::Error::new(Span::call_site(), "kind not found")),
    }
}

fn err_only_supports_fieldless_enums(span: Span) -> syn::Error {
    syn::Error::new(span, "#[derive(Export)] only supports fieldless enums")
}

pub(crate) fn derive_export(input: DeriveInput) -> syn::Result<TokenStream2> {
    let derive_data = parse_derive_input(input)?;

    match derive_data.kind {
        Kind::Enum => {
            let derived_enum = match derive_data.data {
                syn::Data::Enum(data) => data,
                syn::Data::Struct(data) => {
                    return Err(err_only_supports_fieldless_enums(data.struct_token.span()));
                }
                syn::Data::Union(data) => {
                    return Err(err_only_supports_fieldless_enums(data.union_token.span()));
                }
            };
            let export_impl = impl_export(&derive_data.ident, &derived_enum)?;
            Ok(export_impl)
        }
    }
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

    let gdnative_core = crate_gdnative_core();
    let mappings = data
        .variants
        .iter()
        .map(|variant| {
            let key = &variant.ident;
            let val = quote! { #enum_ty::#key as i64 };
            quote! { #gdnative_core::export::hint::EnumHintEntry::with_value(stringify!(#key).to_string(), #val) }
        })
        .collect::<Vec<_>>();

    let impl_block = quote! {
        const _: () = {
            pub enum NoHint {}

            impl #gdnative_core::export::Export for #enum_ty {
                type Hint = NoHint;

                #[inline]
                fn export_info(_hint: Option<Self::Hint>) -> #gdnative_core::export::ExportInfo {
                    let mappings = vec![ #(#mappings),* ];
                    let enum_hint = #gdnative_core::export::hint::EnumHint::with_entries(mappings);
                    return #gdnative_core::export::hint::IntHint::<i64>::Enum(enum_hint).export_info();
                }
            }
        };
    };

    Ok(impl_block)
}

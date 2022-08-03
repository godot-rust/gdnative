use proc_macro2::TokenStream as TokenStream2;
use syn::DeriveInput;

pub(crate) fn derive_export_enum(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let derived_enum = match &input.data {
        syn::Data::Enum(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "#[derive(ExportEnum)] can only use on enum",
            ))
        }
    };

    let to_variant_impl = impl_to_variant(&input.ident, derived_enum)?;
    let from_variant_impl = impl_from_variant(&input.ident, derived_enum)?;
    let export_impl = impl_export(&input.ident, derived_enum)?;
    let combined_impl = quote! {
        #to_variant_impl
        #from_variant_impl
        #export_impl
    };

    Ok(combined_impl)
}

fn impl_to_variant(enum_ty: &syn::Ident, _data: &syn::DataEnum) -> syn::Result<TokenStream2> {
    let impl_block = quote! {
        impl ::gdnative::core_types::ToVariant for #enum_ty {
            #[inline]
            fn to_variant(&self) -> ::gdnative::core_types::Variant {
                (*self as i64).to_variant()
            }
        }
    };

    Ok(impl_block)
}

fn impl_from_variant(enum_ty: &syn::Ident, data: &syn::DataEnum) -> syn::Result<TokenStream2> {
    let as_int = quote! { n };
    let arms = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            if !matches!(variant.fields, syn::Fields::Unit) {
                Err(syn::Error::new(
                    ident.span(),
                    "#[derive(ExportEnum)] only support unit variant",
                ))
            } else {
                Ok(quote! {
                    if #as_int == #enum_ty::#ident as i64 { Ok(#enum_ty::#ident) }
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let impl_block = quote! {
        impl ::gdnative::core_types::FromVariant for #enum_ty {
            #[inline]
            fn from_variant(variant: &::gdnative::core_types::Variant) -> Result<Self, ::gdnative::core_types::FromVariantError> {
                let #as_int = variant.try_to::<i64>()?;
                #(#arms)else *
                // TODO: return FromVariantError
                else {
                    Err(FromVariantError::Unspecified)
                }
            }
        }
    };

    Ok(impl_block)
}

fn impl_export(enum_ty: &syn::Ident, data: &syn::DataEnum) -> syn::Result<TokenStream2> {
    let mappings = data.variants.iter().map(|variant| {
        let key = &variant.ident;
        let val = quote! { #enum_ty::#key as i64 };
        quote! { (stringify!(#key).to_string(), #val) }
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_non_unit_variant() -> syn::Result<()> {
        let named_variant = quote! { NamedVariant { foo: i32, bar: f32 } };
        let unnamed_variant = quote! { UnnamedVariant(String) };
        let input = |variant| {
            parse_quote! {
                pub enum Foo {
                    #variant
                }
            }
        };

        assert!(derive_export_enum(&input(&named_variant)).is_err());
        assert!(derive_export_enum(&input(&unnamed_variant)).is_err());

        Ok(())
    }

    #[test]
    fn deny_struct_derive() -> syn::Result<()> {
        let input = parse_quote! {
            struct Foo;
        };
        assert!(derive_export_enum(&input).is_err());

        Ok(())
    }

    #[test]
    fn deny_union_derive() -> syn::Result<()> {
        let input: DeriveInput = parse_quote! {
            union Foo {
                f1: u32,
                f2: f32,
            }
        };
        assert!(derive_export_enum(&input).is_err());

        Ok(())
    }
}

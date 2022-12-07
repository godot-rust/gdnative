use proc_macro2::{Literal, Span, TokenStream as TokenStream2};

use crate::variant::bounds;
use crate::variant::repr::VariantRepr;
use syn::Ident;

use super::repr::{EnumRepr, EnumReprKind, Repr, StructRepr};
use super::DeriveData;

pub(crate) fn expand_from_variant(derive_data: DeriveData) -> Result<TokenStream2, syn::Error> {
    let DeriveData {
        ident,
        repr,
        mut generics,
    } = derive_data;

    let derived = crate::automatically_derived();

    for param in generics.type_params_mut() {
        param.default = None;
    }

    let input_ident = Ident::new("__variant", Span::call_site());

    let return_expr = match repr {
        Repr::Struct(StructRepr(var_repr)) => {
            let from_variant = var_repr.make_from_variant_expr(&input_ident, &quote! { #ident })?;
            quote! {
                {
                    #from_variant
                }
            }
        }
        Repr::Enum(EnumRepr {
            variants,
            kind,
            primitive_repr,
        }) => match kind {
            EnumReprKind::External => expand_external(&ident, &input_ident, variants)?,
            EnumReprKind::Str => {
                if let Some((var_ident, _)) = variants
                    .iter()
                    .find(|(_, var_repr)| !matches!(var_repr, VariantRepr::Unit(_)))
                {
                    return Err(syn::Error::new(
                        var_ident.span(),
                        "`str` representation can only be used for fieldless enums",
                    ));
                }

                let var_ident_strings: Vec<String> = variants
                    .iter()
                    .map(|(var_ident, _)| format!("{var_ident}"))
                    .collect();

                let var_ident_string_literals = var_ident_strings
                    .iter()
                    .map(|string| Literal::string(string))
                    .collect::<Vec<_>>();

                let ref_var_ident_string_literals = &var_ident_string_literals;

                let variant_idents = variants.iter().map(|(var_ident, _)| var_ident);

                let early_return = variants.is_empty().then(|| {
                    quote! {
                        return Err(FVE::UnknownEnumVariant {
                            variant: __variant,
                            expected: &[],
                        });
                    }
                });

                quote! {
                    let __variant = String::from_variant(#input_ident)?;

                    #early_return

                    match __variant.as_str() {
                        #(
                            #ref_var_ident_string_literals => {
                                Ok(#ident::#variant_idents)
                            },
                        )*
                        variant => Err(FVE::UnknownEnumVariant {
                            variant: variant.to_string(),
                            expected: &[#(#ref_var_ident_string_literals),*],
                        }),
                    }
                }
            }
            EnumReprKind::Repr => {
                let primitive_repr = primitive_repr.ok_or_else(|| {
                    syn::Error::new(
                        ident.span(),
                        "a primitive representation must be specified using `#[repr]`",
                    )
                })?;

                let mut clauses = Vec::new();
                let mut hints = Vec::new();
                let mut discriminant = quote! { 0 };
                for (var_ident, var_repr) in variants.iter() {
                    if let VariantRepr::Unit(expr) = var_repr {
                        if let Some(expr) = expr {
                            discriminant = quote!(#expr);
                        }
                    } else {
                        return Err(syn::Error::new(
                            var_ident.span(),
                            "`repr` representation can only be used for fieldless enums",
                        ));
                    }

                    clauses.push(quote! {
                        if __value == (#discriminant) {
                            return Ok(#ident::#var_ident);
                        }
                    });

                    hints.push(Literal::string(&format!("{var_ident}({discriminant})")));

                    discriminant = quote!(1 + (#discriminant));
                }

                quote! {
                    let __value = <#primitive_repr>::from_variant(#input_ident)?;

                    #(#clauses)*

                    Err(FVE::UnknownEnumVariant {
                        variant: format!("({})", __value),
                        expected: &[#(#hints),*],
                    })
                }
            }
        },
    };

    let generics_no_bounds = bounds::remove_bounds(generics.clone());
    let where_clause = &generics.where_clause;

    let result = quote! {
        #derived
        impl #generics ::gdnative::core_types::FromVariant for #ident #generics_no_bounds #where_clause {
            fn from_variant(
                #input_ident: &::gdnative::core_types::Variant
            ) -> ::std::result::Result<Self, ::gdnative::core_types::FromVariantError> {
                use ::gdnative::core_types::ToVariant;
                use ::gdnative::core_types::FromVariant;
                use ::gdnative::core_types::FromVariantError as FVE;
                use ::gdnative::core_types::VariantEnumRepr;
                use ::gdnative::core_types::VariantStructRepr;

                #return_expr
            }
        }
    };

    Ok(result)
}

fn expand_external(
    ident: &syn::Ident,
    input_ident: &syn::Ident,
    variants: Vec<(Ident, super::repr::VariantRepr)>,
) -> Result<TokenStream2, syn::Error> {
    let var_input_ident = Ident::new("__enum_variant", Span::call_site());

    let var_ident_strings: Vec<String> = variants
        .iter()
        .map(|(var_ident, _)| format!("{var_ident}"))
        .collect();

    let var_ident_string_literals = var_ident_strings
        .iter()
        .map(|string| Literal::string(string))
        .collect::<Vec<_>>();

    let ref_var_ident_string_literals = &var_ident_string_literals;

    let var_from_variants = variants
        .iter()
        .map(|(var_ident, var_repr)| {
            var_repr.make_from_variant_expr(&var_input_ident, &quote! { #ident::#var_ident })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let var_input_ident_iter = std::iter::repeat(&var_input_ident);
    let early_return = variants.is_empty().then(|| {
        quote! {
            return Err(FVE::UnknownEnumVariant {
                variant: __key,
                expected: &[],
            });
        }
    });

    Ok(quote! {
        let __dict = ::gdnative::core_types::Dictionary::from_variant(#input_ident)
            .map_err(|__err| FVE::InvalidEnumRepr {
                expected: VariantEnumRepr::ExternallyTagged,
                error: std::boxed::Box::new(__err),
            })?;
        let __keys = __dict.keys();
        if __keys.len() != 1 {
            return Err(FVE::InvalidEnumRepr {
                expected: VariantEnumRepr::ExternallyTagged,
                error: std::boxed::Box::new(FVE::InvalidLength {
                    expected: 1,
                    len: __keys.len() as usize,
                }),
            })
        }

        let __key = String::from_variant(&__keys.get(0))
        .map_err(|__err| FVE::InvalidEnumRepr {
            expected: VariantEnumRepr::ExternallyTagged,
            error: std::boxed::Box::new(__err),
        })?;

        #early_return

        match __key.as_str() {
            #(
                #ref_var_ident_string_literals => {
                    let #var_input_ident_iter = &__dict.get_or_nil(&__keys.get(0));
                    (#var_from_variants).map_err(|err| FVE::InvalidEnumVariant {
                        variant: #ref_var_ident_string_literals,
                        error: std::boxed::Box::new(err),
                    })
                },
            )*
            variant => Err(FVE::UnknownEnumVariant {
                variant: variant.to_string(),
                expected: &[#(#ref_var_ident_string_literals),*],
            }),
        }
    })
}

use proc_macro2::{Literal, Span, TokenStream as TokenStream2};

use syn::Ident;

use super::repr::Repr;
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
        Repr::Struct(var_repr) => {
            let from_variant = var_repr.from_variant(&input_ident, &quote! { #ident })?;
            quote! {
                {
                    #from_variant
                }
            }
        }
        Repr::Enum(variants) => {
            if variants.is_empty() {
                return Err(syn::Error::new(
                    ident.span(),
                    "cannot derive FromVariant for an uninhabited enum",
                ));
            }

            let var_input_ident = Ident::new("__enum_variant", Span::call_site());

            let var_ident_strings: Vec<String> = variants
                .iter()
                .map(|(var_ident, _)| format!("{}", var_ident))
                .collect();

            let var_ident_string_literals = var_ident_strings
                .iter()
                .map(|string| Literal::string(string))
                .collect::<Vec<_>>();

            let ref_var_ident_string_literals = &var_ident_string_literals;

            let var_from_variants = variants
                .iter()
                .map(|(var_ident, var_repr)| {
                    var_repr.from_variant(&var_input_ident, &quote! { #ident::#var_ident })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let var_input_ident_iter = std::iter::repeat(&var_input_ident);

            quote! {
                {
                    let __dict = ::gdnative::core_types::Dictionary::from_variant(#input_ident)
                        .map_err(|__err| FVE::InvalidEnumRepr {
                            expected: VariantEnumRepr::ExternallyTagged,
                            error: std::boxed::Box::new(__err),
                        })?;

                    let __keys = __dict.keys();
                    if __keys.len() != 1 {
                        Err(FVE::InvalidEnumRepr {
                            expected: VariantEnumRepr::ExternallyTagged,
                            error: std::boxed::Box::new(FVE::InvalidLength {
                                expected: 1,
                                len: __keys.len() as usize,
                            }),
                        })
                    }
                    else {
                        let __key = String::from_variant(&__keys.get(0))
                            .map_err(|__err| FVE::InvalidEnumRepr {
                                expected: VariantEnumRepr::ExternallyTagged,
                                error: std::boxed::Box::new(__err),
                            })?;
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
                    }
                }
            }
        }
    };

    let where_clause = &generics.where_clause;

    let result = quote! {
        #derived
        impl #generics ::gdnative::core_types::FromVariant for #ident #generics #where_clause {
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

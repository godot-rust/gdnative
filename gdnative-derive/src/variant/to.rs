use crate::variant::bounds;
use proc_macro2::{Literal, TokenStream as TokenStream2};

use crate::variant::repr::{EnumReprKind, VariantRepr};

use super::repr::{EnumRepr, Repr, StructRepr};
use super::{DeriveData, ToVariantTrait};

pub(crate) fn expand_to_variant(
    trait_kind: ToVariantTrait,
    derive_data: DeriveData,
) -> Result<TokenStream2, syn::Error> {
    let DeriveData {
        ident,
        repr,
        mut generics,
    } = derive_data;

    let trait_path = trait_kind.trait_path();
    let to_variant_fn = trait_kind.to_variant_fn();
    let to_variant_receiver = trait_kind.to_variant_receiver();
    let derived = crate::automatically_derived();

    for param in generics.type_params_mut() {
        param.default = None;
    }

    let return_expr = match repr {
        Repr::Struct(StructRepr(var_repr)) => {
            let destructure_pattern = var_repr.destructure_pattern();
            let to_variant = var_repr.make_to_variant_expr(trait_kind)?;
            quote! {
                {
                    let #ident #destructure_pattern = self;
                    #to_variant
                }
            }
        }
        Repr::Enum(EnumRepr {
            variants,
            primitive_repr,
            kind,
        }) => {
            if variants.is_empty() {
                quote! {
                    unreachable!("this is an uninhabitable enum");
                }
            } else {
                match kind {
                    EnumReprKind::External => {
                        let match_arms = variants
                            .iter()
                            .map(|(var_ident, var_repr)| {
                                let destructure_pattern = var_repr.destructure_pattern();
                                let to_variant = var_repr.make_to_variant_expr(trait_kind)?;
                                let var_ident_string = format!("{var_ident}");
                                let var_ident_string_literal = Literal::string(&var_ident_string);
                                let tokens = quote! {
                                    #ident::#var_ident #destructure_pattern => {
                                        let __dict = ::gdnative::core_types::Dictionary::new();
                                        let __key = ::gdnative::core_types::ToVariant::to_variant(
                                            &::gdnative::core_types::GodotString::from(#var_ident_string_literal)
                                        );
                                        let __value = #to_variant;
                                        __dict.insert(&__key, &__value);
                                        ::gdnative::core_types::ToVariant::to_variant(&__dict.into_shared())
                                    }
                                };
                                Ok(tokens)
                            })
                            .collect::<Result<Vec<_>, syn::Error>>()?;

                        quote! {
                            match #to_variant_receiver {
                                #( #match_arms ),*
                            }
                        }
                    }
                    EnumReprKind::Str => {
                        let match_arms = variants
                            .iter()
                            .map(|(var_ident, var_repr)| {
                                if !matches!(var_repr, VariantRepr::Unit(_)) {
                                    return Err(syn::Error::new(var_ident.span(), "`str` representation can only be used for fieldless enums"));
                                }

                                let var_ident_string = format!("{var_ident}");
                                let var_ident_string_literal = Literal::string(&var_ident_string);
                                let tokens = quote! {
                                    #ident::#var_ident => {
                                        ::gdnative::core_types::ToVariant::to_variant(#var_ident_string_literal)
                                    }
                                };

                                Ok(tokens)
                            })
                            .collect::<Result<Vec<_>, syn::Error>>()?;

                        quote! {
                            match #to_variant_receiver {
                                #( #match_arms ),*
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

                        if let Some((var_ident, _)) = variants
                            .iter()
                            .find(|(_, var_repr)| !matches!(var_repr, VariantRepr::Unit(_)))
                        {
                            return Err(syn::Error::new(
                                var_ident.span(),
                                "`repr` representation can only be used for fieldless enums",
                            ));
                        }

                        match trait_kind {
                            ToVariantTrait::ToVariant => {
                                quote! {
                                    ::gdnative::core_types::ToVariant::to_variant(&(*self as #primitive_repr))
                                }
                            }
                            ToVariantTrait::OwnedToVariant => {
                                quote! {
                                    ::gdnative::core_types::ToVariant::to_variant(&(self as #primitive_repr))
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let generics_no_bounds = bounds::remove_bounds(generics.clone());
    let where_clause = &generics.where_clause;

    let result = quote! {
        #derived
        impl #generics #trait_path for #ident #generics_no_bounds #where_clause {
            fn #to_variant_fn(#to_variant_receiver) -> ::gdnative::core_types::Variant {
                use #trait_path;
                use ::gdnative::core_types::FromVariant;

                #return_expr
            }
        }
    };

    Ok(result)
}

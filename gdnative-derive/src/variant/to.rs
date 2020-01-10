use proc_macro::TokenStream;
use proc_macro2::Literal;

use super::repr::Repr;
use super::DeriveData;

pub(crate) fn expand_to_variant(derive_data: DeriveData) -> TokenStream {
    let DeriveData {
        ident,
        repr,
        mut generics,
    } = derive_data;

    for param in generics.type_params_mut() {
        param.default = None;
    }

    let return_expr = match repr {
        Repr::Struct(var_repr) => {
            let destructure_pattern = var_repr.destructure_pattern();
            let to_variant = var_repr.to_variant();
            quote! {
                {
                    let #ident #destructure_pattern = &self;
                    #to_variant
                }
            }
        }
        Repr::Enum(variants) => {
            if variants.is_empty() {
                quote! {
                    unreachable!("this is an uninhabitable enum");
                }
            } else {
                let match_arms = variants
                    .iter()
                    .map(|(var_ident, var_repr)| {
                        let destructure_pattern = var_repr.destructure_pattern();
                        let to_variant = var_repr.to_variant();
                        let var_ident_string = format!("{}", var_ident);
                        let var_ident_string_literal = Literal::string(&var_ident_string);
                        quote! {
                            #ident::#var_ident #destructure_pattern => {
                                let mut __dict = ::gdnative::Dictionary::new();
                                let __key = ::gdnative::GodotString::from(#var_ident_string_literal).to_variant();
                                let __value = #to_variant;
                                __dict.set(&__key, &__value);
                                __dict.to_variant()
                            }
                        }
                    });

                quote! {
                    match &self {
                        #( #match_arms ),*
                    }
                }
            }
        }
    };

    let where_clause = &generics.where_clause;

    let result = quote! {
        impl #generics ::gdnative::ToVariant for #ident #generics #where_clause {
            fn to_variant(&self) -> ::gdnative::Variant {
                use ::gdnative::ToVariant;
                use ::gdnative::FromVariant;

                #return_expr
            }
        }
    };

    result.into()
}

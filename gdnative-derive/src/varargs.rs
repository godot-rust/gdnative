use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

use syn::visit::Visit;
use syn::Fields;
use syn::{spanned::Spanned, Data, DeriveInput, Ident};

use crate::utils::extend_bounds::with_visitor;

pub(crate) fn derive_from_varargs(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let derived = crate::automatically_derived();

    if let Data::Struct(struct_data) = input.data {
        let ident = input.ident;

        let mut generics = with_visitor(
            input.generics,
            Some(&syn::parse_quote! { ::gdnative::core_types::FromVariant }),
            None,
            |visitor| {
                visitor.visit_data_struct(&struct_data);
            },
        );

        for param in generics.type_params_mut() {
            param.default = None;
        }

        let input_ident = Ident::new("__args", Span::call_site());
        let where_clause = &generics.where_clause;

        let fields = match &struct_data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(fields) => &fields.unnamed,
            Fields::Unit => {
                return Ok(quote! {
                    #derived
                    impl #generics ::gdnative::export::FromVarargs for #ident #generics #where_clause {
                        fn read<'a>(
                            #input_ident: &mut ::gdnative::export::Varargs<'a>,
                        ) -> std::result::Result<Self, std::vec::Vec<::gdnative::export::ArgumentError<'a>>> {
                            std::result::Result::Ok(#ident)
                        }
                    }
                })
            }
        };

        let mut required = Vec::new();
        let mut optional = Vec::new();
        let mut skipped = Vec::new();
        for field in fields {
            if field.attrs.iter().any(|attr| attr.path.is_ident("skip")) {
                skipped.push(field);
                continue;
            }

            let is_optional = field.attrs.iter().any(|attr| attr.path.is_ident("opt"));
            if !is_optional && !optional.is_empty() {
                return Err(syn::Error::new(
                    field.ident.span(),
                    "cannot add required arguments after optional ones",
                ));
            }
            if is_optional {
                optional.push(field);
            } else {
                required.push(field);
            }
        }

        let req_var_idents = required
            .iter()
            .enumerate()
            .map(|(n, field)| {
                field
                    .ident
                    .clone()
                    .unwrap_or_else(|| Ident::new(&format!("__req_arg_{n}"), Span::call_site()))
            })
            .collect::<Vec<_>>();
        let req_var_names = required
            .iter()
            .map(|field| {
                field.ident.as_ref().map(|id| {
                    let s = id.to_string();
                    quote!(.with_name(#s))
                })
            })
            .collect::<Vec<_>>();
        let req_var_tys = required
            .iter()
            .map(|field| format!("{}", field.ty.to_token_stream()))
            .collect::<Vec<_>>();

        let opt_var_idents = optional
            .iter()
            .enumerate()
            .map(|(n, field)| {
                field
                    .ident
                    .clone()
                    .unwrap_or_else(|| Ident::new(&format!("__opt_arg_{n}"), Span::call_site()))
            })
            .collect::<Vec<_>>();
        let opt_var_names = optional
            .iter()
            .map(|field| {
                field.ident.as_ref().map(|id| {
                    let s = id.to_string();
                    quote!(.with_name(#s))
                })
            })
            .collect::<Vec<_>>();
        let opt_var_tys = optional
            .iter()
            .map(|field| format!("{}", field.ty.to_token_stream()))
            .collect::<Vec<_>>();

        let skipped_var_idents = skipped
            .iter()
            .enumerate()
            .map(|(n, field)| {
                field
                    .ident
                    .clone()
                    .unwrap_or_else(|| Ident::new(&format!("__skipped_arg_{n}"), Span::call_site()))
            })
            .collect::<Vec<_>>();

        Ok(quote! {
            #derived
            impl #generics ::gdnative::export::FromVarargs for #ident #generics #where_clause {
                fn read<'a>(
                    #input_ident: &mut ::gdnative::export::Varargs<'a>,
                ) -> std::result::Result<Self,std::vec:: Vec<::gdnative::export::ArgumentError<'a>>> {
                    let mut __errors = std::vec::Vec::new();

                    #(
                        let #req_var_idents = #input_ident.read()
                            #req_var_names
                            .with_type_name(stringify!(#req_var_tys))
                            .get()
                            .map_err(|err| __errors.push(err))
                            .ok();
                    )*

                    #(
                        let #opt_var_idents = #input_ident.read()
                            #opt_var_names
                            .with_type_name(stringify!(#opt_var_tys))
                            .get_optional()
                            .map_err(|err| __errors.push(err))
                            .ok()
                            .flatten()
                            .unwrap_or_default();
                    )*

                    if !__errors.is_empty() {
                        return std::result::Result::Err(__errors);
                    }

                    #(
                        let #req_var_idents = #req_var_idents.unwrap();
                    )*

                    #(
                        let #skipped_var_idents = core::default::Default::default();
                    )*

                    std::result::Result::Ok(#ident {
                        #(#req_var_idents,)*
                        #(#opt_var_idents,)*
                        #(#skipped_var_idents,)*
                    })
                }
            }
        })
    } else {
        Err(syn::Error::new(
            input.span(),
            "`FromVarargs` can only be derived for structs",
        ))
    }
}

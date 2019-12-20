use proc_macro::TokenStream;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields, Ident, Meta, MetaList, NestedMeta, Path, Type};

mod property_args;
use property_args::{PropertyAttrArgs, PropertyAttrArgsBuilder};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) base: Type,
    pub(crate) register_callback: Option<Path>,
    pub(crate) user_data: Type,
    pub(crate) properties: HashMap<Ident, PropertyAttrArgs>,
}

pub(crate) fn parse_derive_input(input: TokenStream) -> DeriveData {
    let input = match syn::parse_macro_input::parse::<DeriveInput>(input) {
        Ok(val) => val,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let ident = input.ident;

    let inherit_attr = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("inherit"))
        .expect("No \"inherit\" attribute found");

    // read base class
    let base = syn::parse::<Type>(inherit_attr.tokens.clone().into())
        .expect("`inherits` attribute requires the base type as an argument.");

    let register_callback = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("register_with"))
        .map(|attr| {
            attr.parse_args::<Path>()
                .expect("`register_with` attributes requires a function as an argument.")
        });

    let user_data = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("user_data"))
        .map(|attr| {
            syn::parse::<Type>(attr.tokens.clone().into())
                .expect("`userdata` attribute requires a type as an argument.")
        })
        .unwrap_or_else(|| {
            syn::parse::<Type>(quote! { ::gdnative::user_data::DefaultUserData<#ident> }.into())
                .expect("quoted tokens should be a valid type")
        });

    // make sure it's a struct
    let struct_data = if let Data::Struct(data) = input.data {
        data
    } else {
        panic!("NativeClass derive macro only works on structs.");
    };

    // read exported properties
    let properties = if let Fields::Named(names) = &struct_data.fields {
        names
            .named
            .iter()
            .filter_map(|field| {
                let mut property_args = None;

                for attr in field.attrs.iter() {
                    if !attr.path.is_ident("property") {
                        continue;
                    }

                    let meta = attr
                        .parse_meta()
                        .expect("should be able to parse attribute arguments");
                    if let Meta::List(MetaList { nested, .. }) = meta {
                        property_args
                            .get_or_insert_with(PropertyAttrArgsBuilder::default)
                            .extend(nested.iter().map(|arg| match arg {
                                NestedMeta::Meta(Meta::NameValue(ref pair)) => pair,
                                _ => panic!("unexpected argument: {:?}", arg),
                            }));
                    } else {
                        panic!("unexpected meta variant: {:?}", meta);
                    }
                }

                property_args.map(|builder| {
                    let ident = field.ident.clone().expect("fields should be named");
                    (ident, builder.done())
                })
            })
            .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };

    DeriveData {
        name: ident,
        base,
        register_callback,
        user_data,
        properties,
    }
}

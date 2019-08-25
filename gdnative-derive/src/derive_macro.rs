use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields, Ident, Type};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) base: Type,
    pub(crate) user_data: Type,
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
        .find(|a| a.path.segments[0].ident == "inherit")
        .expect("No \"inherit\" attribute found");

    // read base class
    let base = syn::parse::<Type>(inherit_attr.tts.clone().into())
        .expect("`inherits` attribute requires the base type as an argument.");

    let user_data = input
        .attrs
        .iter()
        .find(|a| a.path.segments[0].ident == "user_data")
        .map(|attr| {
            syn::parse::<Type>(attr.tts.clone().into())
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
    if let Fields::Named(_names) = struct_data.fields {
        // TODO
    }

    DeriveData { name: ident, base, user_data }
}

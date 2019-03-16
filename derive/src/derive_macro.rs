use proc_macro::{TokenStream};
use syn::{DeriveInput, Data, Fields, Type, Ident};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) base: Type,
}

pub(crate) fn parse_derive_input(input: TokenStream) -> DeriveData {

    let input = match syn::parse_macro_input::parse::<DeriveInput>(input) {
        Ok(val) => val,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let ident = input.ident;

    let inherit_attr = input.attrs
        .iter()
        .find(|a| a.path.segments[0].ident == "inherit")
        .expect("No \"inherit\" attribute found");

    // read base class
    let base = syn::parse::<Type>(inherit_attr.tts.clone().into())
        .expect("`inherits` attribute requires the base type as an argument.");

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

    DeriveData {
        name: ident,
        base,
    }
}

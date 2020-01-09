use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Generics, Ident};

mod attr;
mod bounds;
mod from;
mod repr;
mod to;

use bounds::extend_bounds;
use repr::{Repr, VariantRepr};

pub(crate) struct DeriveData {
    pub(crate) ident: Ident,
    pub(crate) repr: Repr,
    pub(crate) generics: Generics,
}

pub(crate) fn parse_derive_input(input: TokenStream, bound: &syn::Path) -> DeriveData {
    let input = match syn::parse_macro_input::parse::<DeriveInput>(input) {
        Ok(val) => val,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let repr = match input.data {
        Data::Struct(struct_data) => Repr::Struct(VariantRepr::repr_for(&struct_data.fields)),
        Data::Enum(enum_data) => Repr::Enum(
            enum_data
                .variants
                .iter()
                .map(|variant| {
                    (
                        variant.ident.clone(),
                        VariantRepr::repr_for(&variant.fields),
                    )
                })
                .collect(),
        ),
        Data::Union(_) => panic!("Variant conversion derive macro does not work on unions."),
    };

    let generics = extend_bounds(input.generics, &repr, bound);

    DeriveData {
        ident: input.ident,
        repr,
        generics,
    }
}

pub(crate) fn derive_to_variant(input: TokenStream) -> TokenStream {
    let bound: syn::Path = syn::parse2(quote! { ::gdnative::ToVariant }).unwrap();
    to::expand_to_variant(parse_derive_input(input, &bound))
}

pub(crate) fn derive_from_variant(input: TokenStream) -> TokenStream {
    let bound: syn::Path = syn::parse2(quote! { ::gdnative::FromVariant }).unwrap();
    from::expand_from_variant(parse_derive_input(input, &bound))
}

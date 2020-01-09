extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod methods;
mod native_script;
mod variant;

#[proc_macro_attribute]
pub fn methods(meta: TokenStream, input: TokenStream) -> TokenStream {
    methods::derive_methods(meta, input)
}

#[proc_macro_derive(
    NativeClass,
    attributes(inherit, export, user_data, property, register_with)
)]
pub fn derive_native_class(input: TokenStream) -> TokenStream {
    native_script::derive_native_class(input)
}

#[proc_macro_derive(ToVariant, attributes(variant))]
pub fn derive_to_variant(input: TokenStream) -> TokenStream {
    variant::derive_to_variant(input)
}

#[proc_macro_derive(FromVariant, attributes(variant))]
pub fn derive_from_variant(input: TokenStream) -> TokenStream {
    variant::derive_from_variant(input)
}

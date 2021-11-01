extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod doc;
mod typed_array_element;

#[proc_macro]
pub fn impl_typed_array_element(input: TokenStream) -> TokenStream {
    self::typed_array_element::impl_element(input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

#[proc_macro]
pub fn decl_typed_array_element(input: TokenStream) -> TokenStream {
    self::typed_array_element::decl_element(input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

/*
Leaving code commented-out, as this might be very useful elsewhere

#[proc_macro_attribute]
pub fn doc_variant_collection_safety(attr: TokenStream, item: TokenStream) -> TokenStream {
    self::doc::variant_collection_safety(attr, item)
        .unwrap_or_else(to_compile_errors)
        .into()
}
*/

fn to_compile_errors(error: syn::Error) -> proc_macro2::TokenStream {
    let compile_error = error.to_compile_error();
    quote!(#compile_error)
}

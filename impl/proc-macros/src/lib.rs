extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::AttributeArgs;

mod cfg_ex;
mod doc;
mod pool_array_element;

#[proc_macro]
pub fn impl_typed_array_element(input: TokenStream) -> TokenStream {
    pool_array_element::impl_element(input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

#[proc_macro]
pub fn decl_typed_array_element(input: TokenStream) -> TokenStream {
    pool_array_element::decl_element(input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

/// `#[cfg]` but with custom expansion for GDNative-specific conditional compilation options
#[proc_macro_attribute]
pub fn cfg_ex(meta: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(meta as AttributeArgs);
    let attr = cfg_ex::expand_cfg_ex(args).unwrap_or_else(to_compile_errors);
    let item = proc_macro2::TokenStream::from(item);
    quote!(#attr #item).into()
}

/// `#[cfg_attr]` but with custom expansion for GDNative-specific conditional compilation options
#[proc_macro_attribute]
pub fn cfg_attr_ex(meta: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(meta as AttributeArgs);
    let attr = cfg_ex::expand_cfg_attr_ex(args).unwrap_or_else(to_compile_errors);
    let item = proc_macro2::TokenStream::from(item);
    quote!(#attr #item).into()
}

fn to_compile_errors(error: syn::Error) -> proc_macro2::TokenStream {
    let compile_error = error.to_compile_error();
    quote!(#compile_error)
}

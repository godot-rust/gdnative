use syn::visit_mut::VisitMut;
use syn::{Attribute, ItemFn, ItemImpl};

/*
Leaving code commented-out, as this might be very useful elsewhere

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Item};
pub fn variant_collection_safety(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> Result<TokenStream, syn::Error> {
    let mut item = syn::parse::<Item>(item)?;
    let mut visit = IncludeDocs {
        docs: &[
            "# Safety",
            "",
            "Generally, it's not recommended to mutate variant collections that may be shared. Prefer",
            "`ThreadLocal` or `Unique` collections instead. If you're sure that the current reference",
            "is unique, you may use [`assume_unique`](#method.assume_unique) to convert it to a `Unique`",
            "collection. You may subsequently use [`into_thread_local`](#method.into_thread_local) to",
            "convert it to a `ThreadLocal` one.",
            "",
            "It is only safe to perform operations that may allocate on a shared collection when no",
            "other thread may access the underlying collection during the call.",
        ],
        deprecated: Some(concat!(
            "Care should be used when mutating shared variant collections. Prefer `ThreadLocal` ",
            "or `Unique` collections unless you're absolutely sure that you want this. ",
            "You may use [assume_unique](#method.assume_unique) to convert this to a `Unique` ",
            "collection if you are sure that this is in fact the only reference."
        )),
    };
    visit.visit_item_mut(&mut item);
    Ok(item.to_token_stream())
}
*/

struct IncludeDocs<'a> {
    docs: &'a [&'a str],
    deprecated: Option<&'a str>,
}

impl<'a> IncludeDocs<'a> {
    fn include_docs(&self, attrs: &mut Vec<Attribute>) {
        attrs.extend(self.docs.iter().map(|s| parse_quote!(#[doc=#s])));
        if let Some(s) = self.deprecated {
            attrs.push(parse_quote!(#[deprecated=#s]));
        }
    }
}

impl<'a> VisitMut for IncludeDocs<'a> {
    fn visit_item_fn_mut(&mut self, i: &mut ItemFn) {
        self.include_docs(&mut i.attrs)
    }

    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        self.include_docs(&mut i.attrs)
    }
}

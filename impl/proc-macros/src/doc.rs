use syn::visit_mut::VisitMut;
use syn::{Attribute, ItemFn, ItemImpl};

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

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::AttributeArgs;

pub fn expand_cfg_ex(mut input: AttributeArgs) -> Result<TokenStream, syn::Error> {
    if input.len() != 1 {
        return Err(syn::Error::new(
            Span::call_site(),
            "expecting exactly 1 argument",
        ));
    }

    let mut predicate = input.remove(0);

    let mut visitor = CfgExVisitor::default();
    syn::visit_mut::visit_nested_meta_mut(&mut visitor, &mut predicate);

    let mut it = visitor.errors.into_iter();
    if let Some(mut error) = it.next() {
        for e in it {
            error.combine(e);
        }

        return Err(error);
    }

    Ok(quote!(#[cfg(#predicate)]))
}

pub fn expand_cfg_attr_ex(mut input: AttributeArgs) -> Result<TokenStream, syn::Error> {
    if input.len() < 2 {
        return Err(syn::Error::new(
            Span::call_site(),
            "expecting at least 2 arguments",
        ));
    }

    let mut predicate = input.remove(0);
    let attrs = input;

    let mut visitor = CfgExVisitor::default();
    syn::visit_mut::visit_nested_meta_mut(&mut visitor, &mut predicate);

    let mut it = visitor.errors.into_iter();
    if let Some(mut error) = it.next() {
        for e in it {
            error.combine(e);
        }

        return Err(error);
    }

    Ok(quote!(#[cfg_attr(#predicate, #(#attrs,)*)]))
}

#[derive(Default)]
struct CfgExVisitor {
    errors: Vec<syn::Error>,
}

impl VisitMut for CfgExVisitor {
    fn visit_meta_mut(&mut self, i: &mut syn::Meta) {
        match i {
            syn::Meta::List(list) => self.visit_meta_list_mut(list),
            syn::Meta::NameValue(name_value) => self.visit_meta_name_value_mut(name_value),
            syn::Meta::Path(path) => {
                if !path
                    .segments
                    .first()
                    .map_or(false, |s| s.ident == "gdnative")
                {
                    return;
                }

                if path.segments.len() != 2 {
                    self.errors.push(syn::Error::new(
                        path.span(),
                        "expecting `gdnative::something`",
                    ));
                    return;
                }

                let cfg_name = &path.segments[1].ident;
                let cfg_name_str = cfg_name.to_string();

                match &*cfg_name_str {
                    "inventory_platform_available" => {
                        // https://github.com/mmastrac/rust-ctor/blob/be23064264cdf6239e2b68809762e19094a10fcf/ctor/src/lib.rs#L43
                        *i = parse_quote_spanned!(cfg_name.span() =>
                            any(
                                target_os = "linux",
                                target_os = "android",
                                target_os = "netbsd",
                                target_os = "openbsd",
                                target_os = "dragonfly",
                                target_os = "illumos",
                                target_os = "haiku",
                                target_os = "macos",
                                target_os = "ios",
                                windows,
                            )
                        );
                    }
                    _ => {
                        self.errors.push(syn::Error::new(
                            cfg_name.span(),
                            "unknown gdnative cfg option",
                        ));
                    }
                }
            }
        }
    }
}

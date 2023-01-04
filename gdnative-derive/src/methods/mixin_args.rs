use proc_macro2::{Ident, Span};
use std::fmt::Debug;
use syn::spanned::Spanned;

pub struct MixinArgs {
    pub mixin: Option<MixinKind>,
    pub pub_: bool,
}

#[derive(Debug)]
pub enum MixinKind {
    Auto(Span),
    Named(syn::Ident),
}

pub struct MixinArgsBuilder {
    mixin: Option<MixinKind>,
    pub_: Option<Span>,
}

impl MixinArgsBuilder {
    pub fn new() -> Self {
        Self {
            mixin: None,
            pub_: None,
        }
    }

    /// Error returned when a value are set twice
    /// e.g. #[methods(as = "Foo", as = "Bar")]
    fn err_prop_already_set<T: Debug>(span: Span, prop: &str, old: &T) -> syn::Error {
        syn::Error::new(
            span,
            format!("there is already a '{prop}' attribute with value: {old:?}",),
        )
    }

    // Error returned when the attr value is not a string literal (i.e. not `LitStr`)
    fn err_attr_not_a_string_literal(span: Span, attr: &str) -> syn::Error {
        syn::Error::new(span, format!("'{attr}' value is not a string literal"))
    }

    /// Convert `Lit` to `LitStr`
    fn extract_lit_str(lit: &syn::Lit) -> Option<&syn::LitStr> {
        if let syn::Lit::Str(lit_str) = lit {
            Some(lit_str)
        } else {
            None
        }
    }

    pub fn add_pair(&mut self, pair: &syn::MetaNameValue) -> Result<(), syn::Error> {
        // Update property with input value.
        // Return error when there is already a value set
        macro_rules! update_prop {
            ($prop:ident, $val:expr) => {
                if let Some(old) = self.$prop.replace($val) {
                    return Err(Self::err_prop_already_set(
                        pair.span(),
                        stringify!($prop),
                        &old,
                    ));
                }
            };
        }

        let name = pair
            .path
            .get_ident()
            .expect("should be single identifier")
            .to_string();

        match name.as_str() {
            "mixin" => {
                let name = Self::extract_lit_str(&pair.lit)
                    .ok_or_else(|| Self::err_attr_not_a_string_literal(pair.span(), "path"))?;
                let name = Ident::new(&name.value(), name.span());
                update_prop!(mixin, MixinKind::Named(name));
            }
            _ => {
                return Err(syn::Error::new(
                    pair.span(),
                    format!("unexpected argument: {}", &name),
                ))
            }
        }

        Ok(())
    }

    pub fn add_path(&mut self, path: &syn::Path) -> Result<(), syn::Error> {
        if path.is_ident("pub") {
            if let Some(_span) = self.pub_.replace(path.span()) {
                return Err(Self::err_prop_already_set(path.span(), "pub", &true));
            }
        } else if path.is_ident("mixin") {
            if let Some(kind) = self.mixin.replace(MixinKind::Auto(path.span())) {
                return Err(Self::err_prop_already_set(path.span(), "mixin", &kind));
            }
        } else {
            return Err(syn::Error::new(
                path.span(),
                format!("unexpected argument: {:?}", path.get_ident()),
            ));
        }

        Ok(())
    }
}

impl MixinArgsBuilder {
    pub fn done(self) -> Result<MixinArgs, syn::Error> {
        if let Some(span) = self.pub_ {
            if !matches!(self.mixin, Some(MixinKind::Named(_))) {
                return Err(syn::Error::new(
                    span,
                    "visibility modifiers are only applicable to named mixins",
                ));
            }
        }

        Ok(MixinArgs {
            mixin: self.mixin,
            pub_: self.pub_.is_some(),
        })
    }
}

use std::iter::FromIterator;

use syn::spanned::Spanned;

use super::Direction;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Attr {
    pub skip_to_variant: bool,
    pub skip_from_variant: bool,
    pub to_variant_with: Option<syn::Path>,
    pub from_variant_with: Option<syn::Path>,
}

impl Attr {
    pub(crate) fn skip_bounds(&self, dir: Direction) -> bool {
        match dir {
            Direction::To => self.skip_to_variant,
            Direction::From => self.skip_from_variant,
        }
    }
}

#[derive(Debug, Default)]
pub struct AttrBuilder {
    skip_to_variant: bool,
    skip_from_variant: bool,
    to_variant_with: Option<syn::Path>,
    from_variant_with: Option<syn::Path>,
    errors: Vec<syn::Error>,
}

impl AttrBuilder {
    fn extend_meta(&mut self, meta: &syn::Meta) {
        match meta {
            syn::Meta::Path(flag) => self.set_flag(&flag),
            syn::Meta::NameValue(pair) => self.set_pair(&pair),
            syn::Meta::List(list) => {
                for nested in list.nested.iter() {
                    match nested {
                        syn::NestedMeta::Meta(meta) => self.extend_meta(meta),
                        _ => {
                            self.errors
                                .push(syn::Error::new(nested.span(), "unexpected nested meta"));
                        }
                    }
                }
            }
        }
    }

    fn set_flag(&mut self, flag: &syn::Path) {
        let err = self.try_set_flag(flag).err();
        self.errors.extend(err);
    }

    fn try_set_flag(&mut self, flag: &syn::Path) -> Result<(), syn::Error> {
        let name = flag
            .get_ident()
            .ok_or_else(|| syn::Error::new(flag.span(), "key should be single ident"))?
            .to_string();

        macro_rules! impl_options {
            {
                match $ident:ident . as_str() {
                    $( $name:ident, )*
                }
            } => (
                match $ident.as_str() {
                    $(
                        stringify!($name) => {
                            self.$name = true;
                            return Ok(());
                        },
                    )*
                    _ => {},
                }
            )
        }

        impl_options! {
            match name.as_str() {
                skip_to_variant,
                skip_from_variant,
            }
        }

        #[allow(clippy::single_match)]
        match name.as_str() {
            "skip" => {
                self.skip_to_variant = true;
                self.skip_from_variant = true;
                return Ok(());
            }
            _ => {}
        }

        Err(syn::Error::new(flag.span(), "unknown flag"))
    }

    fn set_pair(&mut self, pair: &syn::MetaNameValue) {
        let err = self.try_set_pair(pair).err();
        self.errors.extend(err);
    }

    #[allow(clippy::single_match)]
    fn try_set_pair(&mut self, pair: &syn::MetaNameValue) -> Result<(), syn::Error> {
        let syn::MetaNameValue { path, lit, .. } = pair;

        let name = path
            .get_ident()
            .ok_or_else(|| syn::Error::new(path.span(), "key should be single ident"))?
            .to_string();

        macro_rules! impl_options {
            {
                match $ident:ident . as_str() = $lit:ident {
                    $( $name:ident: $ty:ty, )*
                }
            } => (
                match $ident.as_str() {
                    $(
                        stringify!($name) => {
                            let val = match $lit {
                                syn::Lit::Str(lit_str) => lit_str.parse::<$ty>()?,
                                _ => return Err(syn::Error::new($lit.span(), "expected string literal")),
                            };

                            if self.$name.replace(val).is_some() {
                                return Err(syn::Error::new($lit.span(), format!(
                                    "the argument {} is already set",
                                    stringify!($name),
                                )));
                            }

                            return Ok(());
                        },
                    )*
                    _ => {},
                }
            )
        }

        impl_options! {
            match name.as_str() = lit {
                to_variant_with: syn::Path,
                from_variant_with: syn::Path,
            }
        }

        match name.as_str() {
            "with" => {
                let path = match lit {
                    syn::Lit::Str(lit_str) => lit_str.parse::<syn::Path>()?,
                    _ => return Err(syn::Error::new(lit.span(), "expected string literal")),
                };

                if self
                    .to_variant_with
                    .replace(parse_quote!(#path::to_variant))
                    .is_some()
                {
                    return Err(syn::Error::new(
                        lit.span(),
                        "the argument to_variant_with is already set",
                    ));
                }

                if self
                    .from_variant_with
                    .replace(parse_quote!(#path::from_variant))
                    .is_some()
                {
                    return Err(syn::Error::new(
                        lit.span(),
                        "the argument from_variant_with is already set",
                    ));
                }

                return Ok(());
            }
            _ => {}
        }

        Err(syn::Error::new(path.span(), "unknown argument"))
    }
}

impl FromIterator<syn::Meta> for AttrBuilder {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = syn::Meta>,
    {
        let mut builder = AttrBuilder::default();
        for meta in iter {
            builder.extend_meta(&meta);
        }
        builder
    }
}

impl AttrBuilder {
    pub fn done(self) -> Result<Attr, Vec<syn::Error>> {
        if self.errors.is_empty() {
            Ok(Attr {
                skip_to_variant: self.skip_to_variant,
                skip_from_variant: self.skip_from_variant,
                to_variant_with: self.to_variant_with,
                from_variant_with: self.from_variant_with,
            })
        } else {
            Err(self.errors)
        }
    }
}

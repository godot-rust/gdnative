use std::iter::FromIterator;

use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::variant::Direction;

use super::AttrBuilder;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FieldAttr {
    pub skip_to_variant: bool,
    pub skip_from_variant: bool,
    pub to_variant_with: Option<syn::Path>,
    pub from_variant_with: Option<syn::Path>,
}

impl FieldAttr {
    pub(crate) fn skip_bounds(&self, dir: Direction) -> bool {
        match dir {
            Direction::To => self.skip_to_variant,
            Direction::From => self.skip_from_variant,
        }
    }
}

#[derive(Debug, Default)]
pub struct FieldAttrBuilder {
    skip_to_variant: bool,
    skip_from_variant: bool,
    to_variant_with: Option<syn::Path>,
    from_variant_with: Option<syn::Path>,
    errors: Vec<syn::Error>,
}

fn generate_error_with_docs(span: Span, message: &str) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "{message}\n\texpecting #[variant(...)]. See documentation:\n\thttps://docs.rs/gdnative/0.9.0/gdnative/core_types/trait.ToVariant.html#field-attributes"
        ),
    )
}

impl FieldAttrBuilder {
    fn extend_meta(&mut self, meta: &syn::Meta) {
        match meta {
            syn::Meta::Path(flag) => self.set_flag(flag),
            syn::Meta::NameValue(pair) => self.set_pair(pair),
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
            .ok_or_else(|| generate_error_with_docs(flag.span(), "Invalid syntax"))?
            .to_string();

        impl_options! {
            self: self,
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

        Err(generate_error_with_docs(
            flag.span(),
            "Missing macro arguments",
        ))
    }

    fn set_pair(&mut self, pair: &syn::MetaNameValue) {
        let err = self.try_set_pair(pair).err();
        self.errors.extend(err);
    }

    #[allow(clippy::single_match)]
    fn try_set_pair(&mut self, pair: &syn::MetaNameValue) -> Result<(), syn::Error> {
        let syn::MetaNameValue { path, lit, .. } = pair;

        const VALID_KEYS: &str =
            "to_variant_with, from_variant_with, with, skip_to_variant, skip_from_variant, skip";

        let name = path
            .get_ident()
            .ok_or_else(|| {
                let path_token = path.segments.iter().enumerate().fold(
                    String::new(),
                    |mut paths, (index, segment)| {
                        if index > 0 {
                            paths.push_str("::");
                        }
                        paths.push_str(&segment.ident.to_string());
                        paths
                    },
                );
                syn::Error::new(
                    path.span(),
                    format!("Found {path_token}, expected one of:\n\t{VALID_KEYS}"),
                )
            })?
            .to_string();

        impl_options! {
            self: self,
            match name.as_str() = lit {
                to_variant_with: syn::Path,
                from_variant_with: syn::Path,
            }
        }

        match name.as_str() {
            "with" => {
                let path = match lit {
                    syn::Lit::Str(lit_str) => lit_str.parse::<syn::Path>()?,
                    _ => {
                        return Err(syn::Error::new(
                            lit.span(),
                            "expecting a path to a module in double quotes: #[variant(with = \"path::to::mod\")]",
                        ))
                    }
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

        Err(syn::Error::new(
            path.span(),
            format!("unknown argument, expected one of:\n\t{VALID_KEYS}"),
        ))
    }
}

impl FromIterator<syn::Meta> for FieldAttrBuilder {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = syn::Meta>,
    {
        let mut builder = FieldAttrBuilder::default();
        for meta in iter {
            builder.extend_meta(&meta);
        }
        builder
    }
}

impl AttrBuilder for FieldAttrBuilder {
    type Attr = FieldAttr;
    fn done(mut self) -> Result<FieldAttr, syn::Error> {
        if self.errors.is_empty() {
            Ok(FieldAttr {
                skip_to_variant: self.skip_to_variant,
                skip_from_variant: self.skip_from_variant,
                to_variant_with: self.to_variant_with,
                from_variant_with: self.from_variant_with,
            })
        } else {
            let first_error = self.errors.remove(0);
            let errors = self
                .errors
                .into_iter()
                .fold(first_error, |mut errors, error| {
                    errors.combine(error);
                    errors
                });

            Err(errors)
        }
    }
}

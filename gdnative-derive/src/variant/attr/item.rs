use std::iter::FromIterator;

use proc_macro2::Span;
use syn::spanned::Spanned;

use crate::variant::{attr::generate_error_with_docs, repr::EnumReprKind};

use super::AttrBuilder;

#[derive(Clone, Debug)]
pub struct ItemAttr {
    pub enum_repr_kind: Option<(EnumReprKind, Span)>,
}

#[derive(Debug, Default)]
pub struct ItemAttrBuilder {
    enum_repr_kind: Option<syn::Ident>,

    errors: Vec<syn::Error>,
}

impl ItemAttrBuilder {
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
        Err(generate_error_with_docs(
            flag.span(),
            "Unknown flag, or missing macro arguments",
        ))
    }

    fn set_pair(&mut self, pair: &syn::MetaNameValue) {
        let err = self.try_set_pair(pair).err();
        self.errors.extend(err);
    }

    #[allow(clippy::single_match)]
    fn try_set_pair(&mut self, pair: &syn::MetaNameValue) -> Result<(), syn::Error> {
        let syn::MetaNameValue { path, lit, .. } = pair;

        const VALID_KEYS: &str = "enum";

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
                "enum" => enum_repr_kind: syn::Ident,
            }
        }

        Err(syn::Error::new(
            path.span(),
            format!("unknown argument, expected one of:\n\t{VALID_KEYS}"),
        ))
    }
}

impl FromIterator<syn::Meta> for ItemAttrBuilder {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = syn::Meta>,
    {
        let mut builder = ItemAttrBuilder::default();
        for meta in iter {
            builder.extend_meta(&meta);
        }
        builder
    }
}

impl AttrBuilder for ItemAttrBuilder {
    type Attr = ItemAttr;
    fn done(mut self) -> Result<ItemAttr, syn::Error> {
        if self.errors.is_empty() {
            let enum_repr_kind = self
                .enum_repr_kind
                .map(|kind| match &*kind.to_string() {
                    "repr" => Ok((EnumReprKind::Repr, kind.span())),
                    "str" => Ok((EnumReprKind::Str, kind.span())),
                    _ => Err(syn::Error::new(
                        kind.span(),
                        "unknown enum representation, expected values: repr, str",
                    )),
                })
                .transpose()?;

            Ok(ItemAttr { enum_repr_kind })
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

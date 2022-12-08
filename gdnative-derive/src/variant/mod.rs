use proc_macro2::TokenStream as TokenStream2;
use syn::{spanned::Spanned, Data, DeriveInput, Generics, Ident, Meta};

mod attr;
mod bounds;
mod from;
mod repr;
mod to;

use bounds::extend_bounds;
use repr::Repr;

use self::{
    attr::{AttrBuilder, ItemAttrBuilder},
    repr::{EnumRepr, StructRepr},
};

pub(crate) struct DeriveData {
    pub(crate) ident: Ident,
    pub(crate) repr: Repr,
    pub(crate) generics: Generics,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Direction {
    To,
    From,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum ToVariantTrait {
    ToVariant,
    OwnedToVariant,
}

impl ToVariantTrait {
    fn trait_path(self) -> syn::Path {
        match self {
            Self::ToVariant => parse_quote! { ::gdnative::core_types::ToVariant },
            Self::OwnedToVariant => parse_quote! { ::gdnative::core_types::OwnedToVariant },
        }
    }

    fn to_variant_fn(self) -> syn::Ident {
        match self {
            Self::ToVariant => parse_quote! { to_variant },
            Self::OwnedToVariant => parse_quote! { owned_to_variant },
        }
    }

    fn to_variant_receiver(self) -> syn::Receiver {
        match self {
            Self::ToVariant => parse_quote! { &self },
            Self::OwnedToVariant => parse_quote! { self },
        }
    }
}

fn improve_meta_error(err: syn::Error) -> syn::Error {
    let error = err.to_string();
    match error.as_str() {
        "expected literal" => {
            syn::Error::new(err.span(), "String expected, wrap with double quotes.")
        }
        other => syn::Error::new(
            err.span(),
            format!("{other}, ie: #[variant(with = \"...\")]"),
        ),
    }
}

fn parse_attrs<'a, A, I>(attrs: I) -> Result<A::Attr, syn::Error>
where
    A: AttrBuilder,
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("variant"))
        .map(|attr| attr.parse_meta().map_err(improve_meta_error))
        .collect::<Result<A, syn::Error>>()?
        .done()
}

pub(crate) fn parse_derive_input(
    input: DeriveInput,
    bound: &syn::Path,
    dir: Direction,
) -> Result<DeriveData, syn::Error> {
    let item_attr = parse_attrs::<ItemAttrBuilder, _>(&input.attrs)?;

    let repr = match input.data {
        Data::Struct(struct_data) => {
            Repr::Struct(StructRepr::repr_for(item_attr, &struct_data.fields)?)
        }
        Data::Enum(enum_data) => {
            let primitive_repr = input.attrs.iter().find_map(|attr| {
                if !attr.path.is_ident("repr") {
                    return None;
                }

                // rustc should do the complaining for us if the `repr` attribute is invalid
                if let Ok(Meta::List(list)) = attr.parse_meta() {
                    list.nested.iter().find_map(|meta| {
                        if let syn::NestedMeta::Meta(Meta::Path(p)) = meta {
                            p.get_ident()
                                .map_or(false, |ident| {
                                    let ident = ident.to_string();
                                    ident.starts_with('u') || ident.starts_with('i')
                                })
                                .then(|| {
                                    syn::Type::Path(syn::TypePath {
                                        qself: None,
                                        path: p.clone(),
                                    })
                                })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            });

            Repr::Enum(EnumRepr::repr_for(item_attr, primitive_repr, &enum_data)?)
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                "Variant conversion derive macro does not work on unions.",
            ))
        }
    };

    let generics = extend_bounds(input.generics, &repr, bound, dir);

    Ok(DeriveData {
        ident: input.ident,
        repr,
        generics,
    })
}

pub(crate) fn derive_to_variant(
    trait_kind: ToVariantTrait,
    input: proc_macro::TokenStream,
) -> Result<TokenStream2, syn::Error> {
    let derive_input = syn::parse_macro_input::parse::<syn::DeriveInput>(input)?;

    let variant = to::expand_to_variant(
        trait_kind,
        parse_derive_input(derive_input, &trait_kind.trait_path(), Direction::To)?,
    )?;

    Ok(variant)
}

pub(crate) fn derive_from_variant(derive_input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let bound: syn::Path = syn::parse_quote! { ::gdnative::core_types::FromVariant };

    let variant = parse_derive_input(derive_input, &bound, Direction::From);
    from::expand_from_variant(variant?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_from_variant_on_uninhabitable_enums() {
        let input = parse_quote! {
            enum Test {}
        };

        derive_from_variant(input).unwrap();
    }
}

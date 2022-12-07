use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use syn::{DataEnum, Fields, Ident, Type};

use super::attr::{FieldAttr, FieldAttrBuilder, ItemAttr};
use super::{parse_attrs, ToVariantTrait};

// Shouldn't matter since this is immediately unpacked anyway.
// Boxing would add too much noise to the match statements.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Repr {
    Struct(StructRepr),
    Enum(EnumRepr),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum VariantRepr {
    Unit(Option<syn::Expr>),
    Struct(Vec<Field>),
    Tuple(Vec<Field>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub attr: FieldAttr,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct StructRepr(pub VariantRepr);

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct EnumRepr {
    pub kind: EnumReprKind,
    pub primitive_repr: Option<Type>,
    pub variants: Vec<(Ident, VariantRepr)>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum EnumReprKind {
    /// Externally-tagged objects, i.e. the original behavior.
    External,
    /// The integer type specified by the `repr` attribute of the enum.
    Repr,
    /// Represent as strings.
    Str,
}

impl EnumRepr {
    pub(crate) fn repr_for(
        attr: ItemAttr,
        primitive_repr: Option<syn::Type>,
        enum_data: &DataEnum,
    ) -> Result<Self, syn::Error> {
        let variants = enum_data
            .variants
            .iter()
            .map(|variant| {
                let mut repr = VariantRepr::repr_for(&variant.fields)?;
                if let VariantRepr::Unit(discriminant) = &mut repr {
                    if let Some((_, expr)) = &variant.discriminant {
                        *discriminant = Some(expr.clone());
                    }
                }

                Ok((variant.ident.clone(), repr))
            })
            .collect::<Result<_, syn::Error>>()?;

        Ok(EnumRepr {
            kind: attr
                .enum_repr_kind
                .map_or(EnumReprKind::External, |(kind, _)| kind),
            primitive_repr,
            variants,
        })
    }
}

impl StructRepr {
    pub(crate) fn repr_for(attr: ItemAttr, fields: &Fields) -> Result<Self, syn::Error> {
        if let Some((_, span)) = attr.enum_repr_kind {
            return Err(syn::Error::new(
                span,
                "`enum` representation can only be set for enums",
            ));
        }

        VariantRepr::repr_for(fields).map(StructRepr)
    }
}

impl VariantRepr {
    pub(crate) fn repr_for(fields: &Fields) -> Result<Self, syn::Error> {
        let this = match fields {
            Fields::Named(fields) => VariantRepr::Struct(
                fields
                    .named
                    .iter()
                    .map(|f| {
                        let ident = f.ident.clone().expect("fields should be named");
                        let ty = f.ty.clone();
                        let attr = parse_attrs::<FieldAttrBuilder, _>(&f.attrs)?;
                        Ok(Field { ident, ty, attr })
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()?,
            ),
            Fields::Unnamed(fields) => VariantRepr::Tuple(
                fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(n, f)| {
                        let ident = Ident::new(&format!("__field_{n}"), Span::call_site());
                        let ty = f.ty.clone();
                        let attr = parse_attrs::<FieldAttrBuilder, _>(&f.attrs)?;
                        Ok(Field { ident, ty, attr })
                    })
                    .collect::<Result<_, syn::Error>>()?,
            ),
            Fields::Unit => VariantRepr::Unit(None),
        };

        Ok(this)
    }

    pub(crate) fn destructure_pattern(&self) -> TokenStream2 {
        match self {
            VariantRepr::Unit(_) => quote! {},
            VariantRepr::Tuple(fields) => {
                let names = fields.iter().map(|f| &f.ident);
                quote! {
                    ( #( #names ),* )
                }
            }
            VariantRepr::Struct(fields) => {
                let names = fields.iter().map(|f| &f.ident);
                quote! {
                    { #( #names ),* }
                }
            }
        }
    }

    pub(crate) fn make_to_variant_expr(
        &self,
        trait_kind: ToVariantTrait,
    ) -> Result<TokenStream2, syn::Error> {
        let tokens = match self {
            VariantRepr::Unit(_) => {
                quote! { ::gdnative::core_types::Dictionary::new().into_shared().to_variant() }
            }
            VariantRepr::Tuple(fields) => {
                if fields.len() == 1 {
                    // as newtype
                    let field = fields.get(0).unwrap();
                    if field.attr.skip_to_variant {
                        return Err(syn::Error::new(
                            field.ident.span(),
                            "cannot skip the only field in a tuple",
                        ));
                    }
                    field.make_to_variant_expr(trait_kind)
                } else {
                    let exprs = fields.iter().filter_map(|f| {
                        if f.attr.skip_to_variant {
                            None
                        } else {
                            Some(f.make_to_variant_expr(trait_kind))
                        }
                    });

                    quote! {
                        {
                            let __array = ::gdnative::core_types::VariantArray::new();
                            #(
                                __array.push(&#exprs);
                            )*
                            __array.into_shared().to_variant()
                        }
                    }
                }
            }
            VariantRepr::Struct(fields) => {
                let fields: Vec<&Field> =
                    fields.iter().filter(|f| !f.attr.skip_to_variant).collect();

                let name_strings: Vec<String> =
                    fields.iter().map(|f| format!("{}", &f.ident)).collect();

                let name_string_literals =
                    name_strings.iter().map(|string| Literal::string(string));

                let exprs = fields.iter().map(|f| f.make_to_variant_expr(trait_kind));

                quote! {
                    {
                        let __dict = ::gdnative::core_types::Dictionary::new();
                        #(
                            {
                                let __key = ::gdnative::core_types::GodotString::from(#name_string_literals).to_variant();
                                __dict.insert(&__key, &#exprs);
                            }
                        )*
                        __dict.into_shared().to_variant()
                    }
                }
            }
        };

        Ok(tokens)
    }

    pub(crate) fn make_from_variant_expr(
        &self,
        variant: &Ident,
        ctor: &TokenStream2,
    ) -> Result<TokenStream2, syn::Error> {
        let tokens = match self {
            VariantRepr::Unit(_) => {
                quote! {
                    if #variant.is_nil() {
                        Err(FVE::InvalidStructRepr {
                            expected: VariantStructRepr::Unit,
                            error: Box::new(FVE::InvalidNil),
                        })
                    }
                    else {
                        Ok(#ctor)
                    }
                }
            }
            VariantRepr::Tuple(fields) => {
                if fields.len() == 1 {
                    // as newtype
                    let field = fields.get(0).unwrap();
                    if field.attr.skip_from_variant {
                        return Err(syn::Error::new(
                            field.ident.span(),
                            "cannot skip the only field in a tuple",
                        ));
                    }
                    let expr = field.make_from_variant_expr(&quote!(#variant));
                    quote! {
                        {
                            #expr.map(#ctor)
                        }
                    }
                } else {
                    let skipped_fields: Vec<&Field> =
                        fields.iter().filter(|f| f.attr.skip_from_variant).collect();

                    let non_skipped_fields: Vec<&Field> = fields
                        .iter()
                        .filter(|f| !f.attr.skip_from_variant)
                        .collect();

                    let skipped_idents = skipped_fields.iter().map(|f| &f.ident);
                    let non_skipped_idents = non_skipped_fields.iter().map(|f| &f.ident);
                    let ctor_idents = fields.iter().map(|f| &f.ident);

                    let expected_len = Literal::usize_suffixed(non_skipped_fields.len());
                    let indices = (0..non_skipped_fields.len() as i32).map(Literal::i32_suffixed);

                    let expr_variant = &quote!(&__array.get(__index));
                    let non_skipped_exprs = non_skipped_fields
                        .iter()
                        .map(|f| f.make_from_variant_expr(expr_variant));

                    quote! {
                        {
                            ::gdnative::core_types::VariantArray::from_variant(#variant)
                                .map_err(|__err| FVE::InvalidStructRepr {
                                    expected: VariantStructRepr::Tuple,
                                    error: std::boxed::Box::new(__err),
                                })
                                .and_then(|__array| {
                                    let __expected = #expected_len;
                                    let __len = __array.len() as usize;
                                    if __len != __expected {
                                        Err(FVE::InvalidLength { expected: __expected, len: __len })
                                    }
                                    else {
                                        #(
                                            let __index = #indices;
                                            let #non_skipped_idents = #non_skipped_exprs
                                                .map_err(|err| FVE::InvalidItem {
                                                    index: __index as usize,
                                                    error: std::boxed::Box::new(err),
                                                })?;
                                        )*
                                        #(
                                            let #skipped_idents = std::default::Default::default();
                                        )*
                                        Ok(#ctor( #(#ctor_idents),* ))
                                    }
                                })
                        }
                    }
                }
            }
            VariantRepr::Struct(fields) => {
                let skipped_fields: Vec<&Field> =
                    fields.iter().filter(|f| f.attr.skip_from_variant).collect();

                let non_skipped_fields: Vec<&Field> = fields
                    .iter()
                    .filter(|f| !f.attr.skip_from_variant)
                    .collect();

                let skipped_idents = skipped_fields.iter().map(|f| &f.ident);
                let non_skipped_idents: Vec<&Ident> =
                    non_skipped_fields.iter().map(|f| &f.ident).collect();
                let ctor_idents = fields.iter().map(|f| &f.ident);

                let name_strings: Vec<String> = non_skipped_idents
                    .iter()
                    .map(|ident| format!("{ident}"))
                    .collect();

                let name_string_literals =
                    name_strings.iter().map(|string| Literal::string(string));

                let expr_variant = &quote!(&__dict.get_or_nil(&__key));
                let exprs = non_skipped_fields
                    .iter()
                    .map(|f| f.make_from_variant_expr(expr_variant));

                quote! {
                    {
                        ::gdnative::core_types::Dictionary::from_variant(#variant)
                            .map_err(|__err| FVE::InvalidStructRepr {
                                expected: VariantStructRepr::Struct,
                                error: std::boxed::Box::new(__err),
                            })
                            .and_then(|__dict| {
                                #(
                                    let __field_name = #name_string_literals;
                                    let __key = ::gdnative::core_types::GodotString::from(__field_name).to_variant();
                                    let #non_skipped_idents = #exprs
                                        .map_err(|err| FVE::InvalidField {
                                            field_name: __field_name,
                                            error: std::boxed::Box::new(err),
                                        })?;
                                )*
                                #(
                                    let #skipped_idents = std::default::Default::default();
                                )*
                                Ok(#ctor { #( #ctor_idents ),* })
                            })
                    }
                }
            }
        };

        Ok(tokens)
    }
}

impl Field {
    fn make_to_variant_expr(&self, trait_kind: ToVariantTrait) -> TokenStream2 {
        let Field { ident, attr, .. } = self;
        if let Some(to_variant_with) = &attr.to_variant_with {
            quote!(#to_variant_with(#ident))
        } else {
            let to_variant_fn = trait_kind.to_variant_fn();
            quote!((#ident).#to_variant_fn())
        }
    }

    fn make_from_variant_expr(&self, variant: &TokenStream2) -> TokenStream2 {
        if let Some(from_variant_with) = &self.attr.from_variant_with {
            quote!(#from_variant_with(#variant))
        } else {
            quote!(::gdnative::core_types::FromVariant::from_variant(#variant))
        }
    }
}

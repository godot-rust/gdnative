use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use syn::{Fields, Ident, Type};

use super::attr::{Attr, AttrBuilder};
use super::ToVariantTrait;

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Repr {
    Struct(VariantRepr),
    Enum(Vec<(Ident, VariantRepr)>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum VariantRepr {
    Unit,
    Struct(Vec<Field>),
    Tuple(Vec<Field>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub attr: Attr,
}

fn improve_meta_error(err: syn::Error) -> syn::Error {
    let error = err.to_string();
    match error.as_str() {
        "expected literal" => {
            syn::Error::new(err.span(), "String expected, wrap with double quotes.")
        }
        other => syn::Error::new(
            err.span(),
            format!("{}, ie: #[variant(with = \"...\")]", other),
        ),
    }
}

fn parse_attrs<'a, I>(attrs: I) -> Result<Attr, syn::Error>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("variant"))
        .map(|attr| attr.parse_meta().map_err(improve_meta_error))
        .collect::<Result<AttrBuilder, syn::Error>>()?
        .done()
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
                        let attr = parse_attrs(&f.attrs)?;
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
                        let ident = Ident::new(&format!("__field_{}", n), Span::call_site());
                        let ty = f.ty.clone();
                        let attr = parse_attrs(&f.attrs)?;
                        Ok(Field { ident, ty, attr })
                    })
                    .collect::<Result<_, syn::Error>>()?,
            ),
            Fields::Unit => VariantRepr::Unit,
        };

        Ok(this)
    }

    pub(crate) fn destructure_pattern(&self) -> TokenStream2 {
        match self {
            VariantRepr::Unit => quote! {},
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

    pub(crate) fn to_variant(
        &self,
        trait_kind: ToVariantTrait,
    ) -> Result<TokenStream2, syn::Error> {
        let tokens = match self {
            VariantRepr::Unit => {
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
                    field.to_variant(trait_kind)
                } else {
                    let exprs = fields.iter().filter_map(|f| {
                        if f.attr.skip_to_variant {
                            None
                        } else {
                            Some(f.to_variant(trait_kind))
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

                let exprs = fields.iter().map(|f| f.to_variant(trait_kind));

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

    pub(crate) fn from_variant(
        &self,
        variant: &Ident,
        ctor: &TokenStream2,
    ) -> Result<TokenStream2, syn::Error> {
        let tokens = match self {
            VariantRepr::Unit => {
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
                    let expr = field.from_variant(&quote!(#variant));
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
                        .map(|f| f.from_variant(expr_variant));

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
                    .map(|ident| format!("{}", ident))
                    .collect();

                let name_string_literals =
                    name_strings.iter().map(|string| Literal::string(string));

                let expr_variant = &quote!(&__dict.get_or_nil(&__key));
                let exprs = non_skipped_fields
                    .iter()
                    .map(|f| f.from_variant(expr_variant));

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
    fn to_variant(&self, trait_kind: ToVariantTrait) -> TokenStream2 {
        let Field { ident, attr, .. } = self;
        if let Some(to_variant_with) = &attr.to_variant_with {
            quote!(#to_variant_with(#ident))
        } else {
            let to_variant_fn = trait_kind.to_variant_fn();
            quote!((#ident).#to_variant_fn())
        }
    }

    fn from_variant(&self, variant: &TokenStream2) -> TokenStream2 {
        if let Some(from_variant_with) = &self.attr.from_variant_with {
            quote!(#from_variant_with(#variant))
        } else {
            quote!(::gdnative::core_types::FromVariant::from_variant(#variant))
        }
    }
}

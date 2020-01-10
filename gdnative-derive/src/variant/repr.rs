use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use syn::{Fields, Ident, Type};

use super::attr::{Attr, AttrBuilder};

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

fn parse_attrs<'a, I>(attrs: I) -> Result<Attr, Vec<syn::Error>>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("variant"))
        .map(|attr| attr.parse_meta())
        .collect::<Result<AttrBuilder, syn::Error>>()
        .map_err(|err| vec![err])?
        .done()
}

impl VariantRepr {
    pub(crate) fn repr_for(fields: &Fields) -> Self {
        match fields {
            Fields::Named(fields) => VariantRepr::Struct(
                fields
                    .named
                    .iter()
                    .map(|f| {
                        let ident = f.ident.clone().expect("fields should be named");
                        let ty = f.ty.clone();
                        let attr =
                            parse_attrs(&f.attrs).expect("should be able to parse attribute");
                        Field { ident, ty, attr }
                    })
                    .collect(),
            ),
            Fields::Unnamed(fields) => VariantRepr::Tuple(
                fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(n, f)| {
                        let ident = Ident::new(&format!("__field_{}", n), Span::call_site());
                        let ty = f.ty.clone();
                        let attr =
                            parse_attrs(&f.attrs).expect("should be able to parse attribute");
                        Field { ident, ty, attr }
                    })
                    .collect(),
            ),
            Fields::Unit => VariantRepr::Unit,
        }
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

    pub(crate) fn to_variant(&self) -> TokenStream2 {
        match self {
            VariantRepr::Unit => {
                quote! { ::gdnative::Dictionary::new().to_variant() }
            }
            VariantRepr::Tuple(fields) => {
                if fields.len() == 1 {
                    // as newtype
                    fields.get(0).unwrap().to_variant()
                } else {
                    let exprs = fields.iter().map(Field::to_variant);
                    quote! {
                        {
                            let mut __array = ::gdnative::VariantArray::new();
                            #(
                                __array.push(&#exprs);
                            )*
                            __array.to_variant()
                        }
                    }
                }
            }
            VariantRepr::Struct(fields) => {
                let names: Vec<&Ident> = fields.iter().map(|f| &f.ident).collect();

                let name_strings: Vec<String> =
                    names.iter().map(|ident| format!("{}", ident)).collect();

                let name_string_literals =
                    name_strings.iter().map(|string| Literal::string(&string));

                let exprs = fields.iter().map(Field::to_variant);

                quote! {
                    {
                        let mut __dict = ::gdnative::Dictionary::new();
                        #(
                            {
                                let __key = ::gdnative::GodotString::from(#name_string_literals).to_variant();
                                __dict.set(&__key, &#exprs);
                            }
                        )*
                        __dict.to_variant()
                    }
                }
            }
        }
    }

    pub(crate) fn from_variant(&self, variant: &Ident, ctor: &TokenStream2) -> TokenStream2 {
        match self {
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
                    let expr = fields.get(0).unwrap().from_variant(&quote!(#variant));
                    quote! {
                        {
                            #expr.map(#ctor)
                        }
                    }
                } else {
                    let types: Vec<&Type> = fields.iter().map(|f| &f.ty).collect();
                    let idents: Vec<&Ident> = fields.iter().map(|f| &f.ident).collect();

                    let decl_idents = idents.iter();
                    let ctor_idents = idents.iter();

                    let self_len = Literal::i32_suffixed(types.len() as i32);
                    let indices = (0..fields.len() as i32).map(|n| Literal::i32_suffixed(n));

                    let expr_variant = &quote!(__array.get_ref(__index));
                    let exprs = fields.iter().map(|f| f.from_variant(expr_variant));

                    quote! {
                        {
                            ::gdnative::VariantArray::from_variant(#variant)
                                .map_err(|__err| FVE::InvalidStructRepr {
                                    expected: VariantStructRepr::Tuple,
                                    error: Box::new(__err),
                                })
                                .and_then(|__array| {
                                    let __expected = #self_len;
                                    let __len = __array.len() as usize;
                                    if __len != __expected {
                                        Err(FVE::InvalidLength { expected: __expected, len: __len })
                                    }
                                    else {
                                        #(
                                            let __index = #indices;
                                            let #decl_idents = #exprs
                                                .map_err(|err| FromVariantError::InvalidItem {
                                                    index: __index as usize,
                                                    error: Box::new(err),
                                                })?;
                                        )*
                                        Ok(#ctor( #(#ctor_idents),* ))
                                    }
                                })
                        }
                    }
                }
            }
            VariantRepr::Struct(fields) => {
                let names: Vec<&Ident> = fields.iter().map(|f| &f.ident).collect();

                let name_strings: Vec<String> =
                    names.iter().map(|ident| format!("{}", ident)).collect();

                let name_string_literals =
                    name_strings.iter().map(|string| Literal::string(&string));

                let decl_idents = names.iter();
                let ctor_idents = names.iter();

                let expr_variant = &quote!(__dict.get_ref(&__key));
                let exprs = fields.iter().map(|f| f.from_variant(expr_variant));

                quote! {
                    {
                        ::gdnative::Dictionary::from_variant(#variant)
                            .map_err(|__err| FVE::InvalidStructRepr {
                                expected: VariantStructRepr::Struct,
                                error: Box::new(__err),
                            })
                            .and_then(|__dict| {
                                #(
                                    let __field_name = #name_string_literals;
                                    let __key = ::gdnative::GodotString::from(__field_name).to_variant();
                                    let #decl_idents = #exprs
                                        .map_err(|err| FVE::InvalidField {
                                            field_name: __field_name,
                                            error: Box::new(err),
                                        })?;
                                )*
                                Ok(#ctor { #( #ctor_idents ),* })
                            })
                    }
                }
            }
        }
    }
}

impl Field {
    fn to_variant(&self) -> TokenStream2 {
        let Field { ident, attr, .. } = self;
        if let Some(to_variant_with) = &attr.to_variant_with {
            quote!(#to_variant_with(#ident))
        } else {
            quote!((#ident).to_variant())
        }
    }

    fn from_variant(&self, variant: &TokenStream2) -> TokenStream2 {
        if let Some(from_variant_with) = &self.attr.from_variant_with {
            quote!(#from_variant_with(#variant))
        } else {
            quote!(FromVariant::from_variant(#variant))
        }
    }
}

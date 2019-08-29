use std::collections::{HashSet};

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields, Ident, Type, TypePath, Generics};
use syn::visit::{self, Visit};
use proc_macro2::{TokenStream as TokenStream2, Span, Literal};

pub(crate) struct DeriveData {
    pub(crate) ident: Ident,
    pub(crate) repr: Repr,
    pub(crate) generics: Generics,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum Repr {
    Struct(VariantRepr),
    Enum(Vec<(Ident, VariantRepr)>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum VariantRepr {
    Unit,
    Struct(Vec<(Ident, Type)>),
    Tuple(Vec<(Ident, Type)>),
}

impl VariantRepr {
    fn repr_for(fields: &Fields) -> Self {
        match fields {
            Fields::Named(fields) => {
                VariantRepr::Struct(fields.named
                    .iter()
                    .map(|f| (f.ident.clone().expect("fields should be named"), f.ty.clone()))
                    .collect())
            },
            Fields::Unnamed(fields) => {
                VariantRepr::Tuple(fields.unnamed
                    .iter()
                    .enumerate()
                    .map(|(n, f)| (Ident::new(&format!("__field_{}", n), Span::call_site()), f.ty.clone()))
                    .collect())
            },
            Fields::Unit => {
                VariantRepr::Unit
            },
        }
    }

    fn destructure_pattern(&self) -> TokenStream2 {
        match self {
            VariantRepr::Unit => quote! { },
            VariantRepr::Tuple(fields) => {
                let names = fields
                    .iter()
                    .map(|(ident, _)| ident);
                quote! {
                    ( #( #names ),* )
                }
            },
            VariantRepr::Struct(fields) => {
                let names = fields
                    .iter()
                    .map(|(ident, _)| ident);
                quote! {
                    { #( #names ),* }
                }
            },
        }
    }

    fn to_variant(&self) -> TokenStream2 {
        match self {
            VariantRepr::Unit => {
                quote! { ::gdnative::Dictionary::new().to_variant() }
            },
            VariantRepr::Tuple(fields) => {
                let names: Vec<&Ident> = fields
                    .iter()
                    .map(|(ident, _)| ident)
                    .collect();

                if names.len() == 1 {
                    // as newtype
                    let inner = names.get(0).unwrap();
                    quote! {
                        #inner.to_variant()
                    }
                }
                else {
                    quote! {
                        {
                            let mut __array = ::gdnative::VariantArray::new();
                            #(
                                __array.push(&(#names).to_variant());
                            )*
                            __array.to_variant()
                        }
                    }
                }
            },
            VariantRepr::Struct(fields) => {
                let names: Vec<&Ident> = fields
                    .iter()
                    .map(|(ident, _)| ident)
                    .collect();

                let name_strings: Vec<String> = names
                    .iter()
                    .map(|ident| format!("{}", ident))
                    .collect();

                let name_string_literals = name_strings
                    .iter()
                    .map(|string| Literal::string(&string));

                quote! {
                    {
                        let mut __dict = ::gdnative::Dictionary::new();
                        #(
                            {
                                let __key = ::gdnative::GodotString::from(#name_string_literals).to_variant();
                                __dict.set(&__key, &(#names).to_variant());
                            }
                        )*
                        __dict.to_variant()
                    }
                }
            },
        }
    }

    fn from_variant(&self, variant: &Ident, ctor: &TokenStream2) -> TokenStream2 {
        match self {
            VariantRepr::Unit => {
                quote! {
                    if #variant.is_nil() {
                        None
                    }
                    else {
                        Some(#ctor)
                    }
                }
            },
            VariantRepr::Tuple(fields) => {
                let types: Vec<&Type> = fields
                    .iter()
                    .map(|(_, ty)| ty)
                    .collect();

                if types.len() == 1 {
                    // as newtype
                    let inner = types.get(0).unwrap();
                    quote! {
                        {
                            let __inner = #inner::from_variant(#variant)?;
                            Some(#ctor(__inner))
                        }
                    }
                }
                else {
                    let idents: Vec<&Ident> = fields
                        .iter()
                        .map(|(ident, _)| ident)
                        .collect();
                        
                    let decl_idents = idents.iter();
                    let ctor_idents = idents.iter();

                    let self_len = Literal::i32_suffixed(types.len() as i32);
                    let indices = (0..fields.len() as i32)
                        .map(|n| Literal::i32_suffixed(n));

                    quote! {
                        {
                            let __array = #variant.try_to_array()?;
                            if __array.len() != #self_len {
                                None
                            }
                            else {
                                #(
                                    let #decl_idents = FromVariant::from_variant(__array.get_ref(#indices))?;
                                )*
                                Some(#ctor( #(#ctor_idents),* ))
                            }
                        }
                    }
                }
            },
            VariantRepr::Struct(fields) => {
                let names: Vec<&Ident> = fields
                    .iter()
                    .map(|(ident, _)| ident)
                    .collect();

                let name_strings: Vec<String> = names
                    .iter()
                    .map(|ident| format!("{}", ident))
                    .collect();

                let name_string_literals = name_strings
                    .iter()
                    .map(|string| Literal::string(&string));
                        
                let decl_idents = names.iter();
                let ctor_idents = names.iter();

                quote! {
                    {
                        let __dict = #variant.try_to_dictionary()?;
                        #(
                            let __key = ::gdnative::GodotString::from(#name_string_literals).to_variant();
                            let #decl_idents = FromVariant::from_variant(__dict.get_ref(&__key))?;
                        )*
                        Some(#ctor { #( #ctor_idents ),* })
                    }
                }
            },
        }
    }
}

pub(crate) fn extend_bounds(generics: Generics, repr: &Repr, bound: &syn::Path) -> Generics {
    
    // recursively visit all the field types to find what types should be bounded
    struct Visitor<'ast> {
        all_type_params: HashSet<Ident>,
        used: HashSet<&'ast TypePath>,
    }

    impl<'ast> Visit<'ast> for Visitor<'ast> {
        fn visit_type_path(&mut self, type_path: &'ast TypePath) {
            let path = &type_path.path;
            if let Some(seg) = path.segments.last() {
                if seg.value().ident == "PhantomData" {
                    // things inside PhantomDatas doesn't need to be bounded, so stopping
                    // recursive visit here
                    return;
                }
            }
            if let Some(seg) = path.segments.first() {
                if self.all_type_params.contains(&seg.value().ident) {
                    // if the first segment of the type path is a known type variable, then this
                    // is likely an associated type
                    // TODO: what about cases like <Foo<T> as Trait>::A? Maybe too fringe to be
                    // useful? serde_derive can't seem to parse these either. Probably good enough.
                    self.used.insert(type_path);
                }
            }
            visit::visit_path(self, &type_path.path);
        }
    }

    let all_type_params = generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect();

    let mut visitor = Visitor {
        all_type_params: all_type_params,
        used: HashSet::new(),
    };

    // iterate through parsed variant representations and visit the types of each field
    fn visit_var_repr<'ast>(visitor: &mut Visitor<'ast>, repr: &'ast VariantRepr) {
        match repr {
            VariantRepr::Unit => { },
            VariantRepr::Tuple(tys) => {
                for (_, ty) in tys.iter() {
                    visitor.visit_type(ty);
                }
            },
            VariantRepr::Struct(fields) => {
                for (_, ty) in fields.iter() {
                    visitor.visit_type(ty);
                }
            }
        }
    }

    match repr {
        Repr::Enum(ref variants) => {
            for (_, var_repr) in variants.iter() {
                visit_var_repr(&mut visitor, var_repr);
            }
        },
        Repr::Struct(var_repr) => {
            visit_var_repr(&mut visitor, var_repr);
        },
    }

    // where thing: is_trait
    fn where_predicate(thing: Type, is_trait: syn::Path) -> syn::WherePredicate {
        syn::WherePredicate::Type(syn::PredicateType {
            lifetimes: None,
            bounded_ty: thing,
            colon_token: <Token![:]>::default(),
            bounds: vec![syn::TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: is_trait,
            })]
            .into_iter()
            .collect(),
        })
    }

    // place bounds on all used type parameters and associated types
    let new_predicates = visitor.used
        .into_iter()
        .cloned()
        .map(|bounded_ty| {
            where_predicate(syn::Type::Path(bounded_ty), bound.clone())
        });

    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .extend(new_predicates);

    generics
}

pub(crate) fn parse_derive_input(input: TokenStream, bound: &syn::Path) -> DeriveData {
    let input = match syn::parse_macro_input::parse::<DeriveInput>(input) {
        Ok(val) => val,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let repr = match input.data {
        Data::Struct(struct_data) => Repr::Struct(VariantRepr::repr_for(&struct_data.fields)),
        Data::Enum(enum_data) => {
            Repr::Enum(enum_data.variants
                .iter()
                .map(|variant| (variant.ident.clone(), VariantRepr::repr_for(&variant.fields)))
                .collect())
        },
        Data::Union(_) => panic!("Variant conversion derive macro does not work on unions."),
    };

    let generics = extend_bounds(input.generics, &repr, bound);

    DeriveData { ident: input.ident, repr, generics }
}

pub(crate) fn derive_to_variant(input: TokenStream) -> TokenStream {
    let bound: syn::Path = syn::parse2(quote! { ::gdnative::ToVariant }).unwrap();
    let DeriveData { ident, repr, generics } = parse_derive_input(input, &bound);

    let return_expr = match repr {
        Repr::Struct(var_repr) => {
            let destructure_pattern = var_repr.destructure_pattern();
            let to_variant = var_repr.to_variant();
            quote! {
                {
                    let #ident #destructure_pattern = &self;
                    #to_variant
                }
            }
        },
        Repr::Enum(variants) => {
            let match_arms = variants
                .iter()
                .map(|(var_ident, var_repr)| {
                    let destructure_pattern = var_repr.destructure_pattern();
                    let to_variant = var_repr.to_variant();
                    let var_ident_string = format!("{}", var_ident);
                    let var_ident_string_literal = Literal::string(&var_ident_string);
                    quote! {
                        #ident::#var_ident #destructure_pattern => {
                            let mut __dict = ::gdnative::Dictionary::new();
                            let __key = ::gdnative::GodotString::from(#var_ident_string_literal).to_variant();
                            let __value = #to_variant;
                            __dict.set(&__key, &__value);
                            __dict.to_variant()
                        }
                    }
                });

            quote! {
                match &self {
                    #( #match_arms ),*
                }
            }
        },
    };

    let where_clause = &generics.where_clause;

    let result = quote! {
        impl #generics ::gdnative::ToVariant for #ident #generics #where_clause {
            fn to_variant(&self) -> ::gdnative::Variant {
                use ::gdnative::ToVariant;
                use ::gdnative::FromVariant;

                #return_expr
            }
        }
    };

    result.into()
}

pub(crate) fn derive_from_variant(input: TokenStream) -> TokenStream {
    let bound: syn::Path = syn::parse2(quote! { ::gdnative::FromVariant }).unwrap();
    let DeriveData { ident, repr, generics } = parse_derive_input(input, &bound);

    let input_ident = Ident::new("__variant", Span::call_site());

    let return_expr = match repr {
        Repr::Struct(var_repr) => {
            let from_variant = var_repr.from_variant(&input_ident, &quote! { #ident });
            quote! {
                {
                    #from_variant
                }
            }
        },
        Repr::Enum(variants) => {
            let var_input_ident = Ident::new("__enum_variant", Span::call_site());

            let var_ident_strings: Vec<String> = variants
                .iter()
                .map(|(var_ident, _)| format!("{}", var_ident))
                .collect();

            let var_ident_string_literals = var_ident_strings
                .iter()
                .map(|string| Literal::string(&string));

            let var_from_variants = variants
                .iter()
                .map(|(var_ident, var_repr)| {
                    var_repr.from_variant(&var_input_ident, &quote! { #ident::#var_ident })
                });

            let var_input_ident_iter = std::iter::repeat(&var_input_ident);

            quote! {
                {
                    let __dict = #input_ident.try_to_dictionary()?;
                    let __keys = __dict.keys();
                    if __keys.len() != 1 {
                        None
                    }
                    else {
                        let __key = __keys.get_ref(0).try_to_string()?;
                        match __key.as_str() {
                            #(
                                #var_ident_string_literals => {
                                    let #var_input_ident_iter = __dict.get_ref(__keys.get_ref(0));
                                    #var_from_variants
                                },
                            )*
                            _ => None,
                        }
                    }
                }
            }
        },
    };

    let where_clause = &generics.where_clause;

    let result = quote! {
        impl #generics ::gdnative::FromVariant for #ident #generics #where_clause {
            fn from_variant(#input_ident: &::gdnative::Variant) -> ::std::option::Option<Self> {
                use ::gdnative::ToVariant;
                use ::gdnative::FromVariant;

                #return_expr
            }
        }
    };

    result.into()
}
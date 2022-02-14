use proc_macro2::TokenStream as TokenStream2;

use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, Fields, Ident, Meta, MetaList, NestedMeta, Path, Stmt, Type};

mod property_args;
use property_args::{PropertyAttrArgs, PropertyAttrArgsBuilder, PropertyGet, PropertySet};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) godot_name: Option<String>,
    pub(crate) base: Type,
    pub(crate) register_callback: Option<Path>,
    pub(crate) user_data: Type,
    pub(crate) properties: Vec<(Ident, PropertyAttrArgs)>,
    pub(crate) no_constructor: bool,
}

pub(crate) fn impl_empty_nativeclass(derive_input: &DeriveInput) -> TokenStream2 {
    let derived = crate::automatically_derived();
    let name = &derive_input.ident;

    let maybe_statically_named = if derive_input.generics.params.is_empty() {
        let name_str = name.to_string();
        Some(quote! {
            #derived
            impl ::gdnative::export::StaticallyNamed for #name {
                const CLASS_NAME: &'static str = #name_str;
            }
        })
    } else {
        None
    };

    quote! {
        #derived
        impl ::gdnative::export::NativeClass for #name {
            type Base = ::gdnative::api::Object;
            type UserData = ::gdnative::export::user_data::LocalCellData<Self>;

            fn init(owner: ::gdnative::object::TRef<'_, Self::Base, Shared>) -> Self {
                unimplemented!()
            }
        }

        #maybe_statically_named
    }
}

pub(crate) fn derive_native_class(derive_input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let derived = crate::automatically_derived();
    let data = parse_derive_input(derive_input)?;

    // generate NativeClass impl
    let trait_impl = {
        let name = data.name;
        let base = data.base;
        let user_data = data.user_data;
        let register_callback = data
            .register_callback
            .map(|function_path| quote!(#function_path(builder);))
            .unwrap_or(quote!({}));
        let properties = data
            .properties
            .into_iter()
            .map(|(ident, config)| {
                let with_default = config
                    .default
                    .as_ref()
                    .map(|default_value| quote!(.with_default(#default_value)));
                let with_hint = config
                    .hint
                    .as_ref()
                    .map(|hint_fn| quote!(.with_hint(#hint_fn())))
                    .or_else(|| {
                        if let Some(Some(prefix)) = &config.group {
                            Some(quote!(.with_hint(::gdnative::export::hint::GroupHint::new(#prefix))))
                        } else {
                            None
                        }
                    });

                let with_usage = if config.no_editor {
                    Some(quote!(.with_usage(::gdnative::export::PropertyUsage::NOEDITOR)))
                } else if config.group.is_some() {
                    Some(quote!(.with_usage({
                        ::gdnative::export::PropertyUsage::DEFAULT
                        | ::gdnative::export::PropertyUsage::GROUP
                    })))
                } else if config.category {
                    Some(quote!(.with_usage({
                        ::gdnative::export::PropertyUsage::DEFAULT
                        | ::gdnative::export::PropertyUsage::CATEGORY
                    })))
                } else {
                    None
                };
                // check whether this property type is `Property<T>`. if so, extract T from it.
                let property_ty = match config.ty {
                    Type::Path(ref path) => path
                        .path
                        .segments
                        .iter()
                        .last()
                        .filter(|seg| seg.ident == "Property")
                        .and_then(|seg| match seg.arguments {
                            syn::PathArguments::AngleBracketed(ref params) => params.args.first(),
                            _ => None,
                        })
                        .and_then(|arg| match arg {
                            syn::GenericArgument::Type(ref ty) => Some(ty),
                            _ => None,
                        })
                        .map(|ty| quote!(::<#ty>)),
                    _ => None,
                };

                // Attribute is #[property] (or has other arguments which are not relevant here)
                let is_standalone_attribute = config.get.is_none() && config.set.is_none();
                // Attribute is #[property(get)] or #[property(get, set="path")]
                let has_default_getter = matches!(config.get, Some(PropertyGet::Default));
                // Attribute is #[property(set)] or #[property(get="path", set)]
                let has_default_setter = matches!(config.set, Some(PropertySet::Default));

                // Field type is `Property<T>`
                if property_ty.is_some()
                    && (is_standalone_attribute || has_default_getter || has_default_setter)
                {
                    return Err(syn::Error::new(
                        ident.span(),
                        "The `#[property]` attribute requires explicit paths for `get` and `set` argument; \
                        the defaults #[property], #[property(get)] and #[property(set)] are not allowed."
                    ));
                }

                // if both of them are not set, i.e. `#[property]`. implicitly use both getter/setter
                let (get, set) = if is_standalone_attribute {
                    (Some(PropertyGet::Default), Some(PropertySet::Default))
                } else {
                    (config.get, config.set)
                };
                let before_get: Option<Stmt> = config
                    .before_get
                    .map(|path_expr| parse_quote!(#path_expr(this, _owner);));
                let after_get: Option<Stmt> = config
                    .after_get
                    .map(|path_expr| parse_quote!(#path_expr(this, _owner);));
                let with_getter = get.map(|get| {
                    let register_fn = match get {
                        PropertyGet::Owned(_) => quote!(with_getter),
                        _ => quote!(with_ref_getter),
                    };
                    let get: Expr = match get {
                        PropertyGet::Default => parse_quote!(&this.#ident),
                        PropertyGet::Owned(path_expr) | PropertyGet::Ref(path_expr) => parse_quote!(#path_expr(this, _owner))
                    };
                    quote!(
                        .#register_fn(|this: &#name, _owner: ::gdnative::object::TRef<Self::Base>| {
                            #before_get
                            let res = #get;
                            #after_get
                            res
                        })
                    )
                });
                let before_set: Option<Stmt> = config
                    .before_set
                    .map(|path_expr| parse_quote!(#path_expr(this, _owner);));
                let after_set: Option<Stmt> = config
                    .after_set
                    .map(|path_expr| parse_quote!(#path_expr(this, _owner);));
                let with_setter = set.map(|set| {
                    let set: Stmt = match set {
                        PropertySet::Default => parse_quote!(this.#ident = v;),
                        PropertySet::WithPath(path_expr) => parse_quote!(#path_expr(this, _owner, v);),
                    };
                    quote!(
                    .with_setter(|this: &mut #name, _owner: ::gdnative::object::TRef<Self::Base>, v| {
                        #before_set
                        #set
                        #after_set
                    }))
                });

                let label = config.path.unwrap_or_else(|| format!("{}", ident));
                Ok(quote!({
                    builder.property#property_ty(#label)
                        #with_default
                        #with_hint
                        #with_usage
                        #with_getter
                        #with_setter
                        .done();
                }))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let maybe_statically_named = data.godot_name.map(|name_str| {
            quote! {
                #derived
                impl ::gdnative::export::StaticallyNamed for #name {
                    const CLASS_NAME: &'static str = #name_str;
                }
            }
        });

        let init = if data.no_constructor {
            None
        } else {
            Some(quote! {
                fn init(owner: ::gdnative::object::TRef<Self::Base>) -> Self {
                    Self::new(::gdnative::export::OwnerArg::from_safe_ref(owner))
                }
            })
        };

        quote!(
            #derived
            impl ::gdnative::export::NativeClass for #name {
                type Base = #base;
                type UserData = #user_data;

                #init

                fn register_properties(builder: &::gdnative::export::ClassBuilder<Self>) {
                    #(#properties)*;
                    #register_callback
                }
            }

            #maybe_statically_named
        )
    };

    // create output token stream
    Ok(trait_impl)
}

fn parse_derive_input(input: &DeriveInput) -> Result<DeriveData, syn::Error> {
    let span = proc_macro2::Span::call_site();

    let ident = input.ident.clone();

    let inherit_attr = input.attrs.iter().find(|a| a.path.is_ident("inherit"));

    // read base class
    let base = if let Some(attr) = inherit_attr {
        attr.parse_args::<Type>()?
    } else {
        syn::parse2::<Type>(quote! { ::gdnative::api::Reference }).unwrap()
    };

    let godot_name = if input.generics.params.is_empty() {
        Some(ident.to_string())
    } else {
        None
    };

    let register_callback = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("register_with"))
        .map(|attr| attr.parse_args::<Path>())
        .transpose()?;

    let user_data = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("user_data"))
        .map(|attr| attr.parse_args::<Type>())
        .unwrap_or_else(|| {
            Ok(syn::parse2::<Type>(
                quote! { ::gdnative::export::user_data::DefaultUserData<#ident> },
            )
            .expect("quoted tokens for default userdata should be a valid type"))
        })?;

    let no_constructor = input
        .attrs
        .iter()
        .any(|a| a.path.is_ident("no_constructor"));

    // make sure it's a struct
    let struct_data = if let Data::Struct(data) = &input.data {
        data
    } else {
        return Err(syn::Error::new(
            span,
            "NativeClass derive macro only works on structs.",
        ));
    };

    // Find all fields with a `#[property]` attribute
    let mut properties = Vec::new();

    if let Fields::Named(names) = &struct_data.fields {
        for field in &names.named {
            let mut property_args = None;

            for attr in field.attrs.iter() {
                if !attr.path.is_ident("property") {
                    continue;
                }

                let meta = attr.parse_meta()?;

                match meta {
                    Meta::List(MetaList { nested, .. }) => {
                        let attr_args_builder = property_args
                            .get_or_insert_with(|| PropertyAttrArgsBuilder::new(&field.ty));

                        for arg in &nested {
                            if let NestedMeta::Meta(Meta::NameValue(ref pair)) = arg {
                                attr_args_builder.add_pair(pair)?;
                            } else if let NestedMeta::Meta(Meta::Path(ref path)) = arg {
                                attr_args_builder.add_path(path)?;
                            } else {
                                let msg = format!("Unexpected argument: {:?}", arg);
                                return Err(syn::Error::new(arg.span(), msg));
                            }
                        }
                    }
                    Meta::Path(_) => {
                        property_args
                            .get_or_insert_with(|| PropertyAttrArgsBuilder::new(&field.ty));
                    }
                    m => {
                        let msg = format!("Unexpected meta variant: {:?}", m);
                        return Err(syn::Error::new(m.span(), msg));
                    }
                }
            }

            if let Some(builder) = property_args {
                let ident = field
                    .ident
                    .clone()
                    .ok_or_else(|| syn::Error::new(field.ident.span(), "Fields should be named"))?;
                properties.push((ident, builder.done()));
            }
        }
    };

    Ok(DeriveData {
        name: ident,
        godot_name,
        base,
        register_callback,
        user_data,
        properties,
        no_constructor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_property() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property]
                bar: String,
            }"#,
        )
        .unwrap();

        let input: DeriveInput = syn::parse2(input).unwrap();

        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_before_get() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property(before_get = "foo::bar")]
                bar: String,
            }"#,
        )
        .unwrap();

        let input: DeriveInput = syn::parse2(input).unwrap();

        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_before_get_err() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property(before_get = "foo::bar")]
                bar: String,
            }"#,
        )
        .unwrap();

        let input: DeriveInput = syn::parse2(input).unwrap();

        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_no_editor() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property(no_editor)]
                bar: String,
            }"#,
        )
        .unwrap();

        let input: DeriveInput = syn::parse2(input).unwrap();

        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_get_set() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property(get = "get_bar", set = "set_bar")]
                bar: i64,
            }"#,
        )
        .unwrap();
        let input: DeriveInput = syn::parse2(input).unwrap();
        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_default_get_set() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property(get, set)]
                bar: i64,
            }"#,
        )
        .unwrap();
        let input: DeriveInput = syn::parse2(input).unwrap();
        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_default_get_ref() {
        let input: TokenStream2 = syn::parse_str(
            r#"
            #[inherit(Node)]
            struct Foo {
                #[property(get_ref = "Self::get_bar")]
                bar: i64,
            }"#,
        )
        .unwrap();
        let input: DeriveInput = syn::parse2(input).unwrap();
        parse_derive_input(&input).unwrap();
    }

    #[test]
    fn derive_property_combinations() {
        let attr_none = quote! {       #[property]                          };
        let attr_get = quote! {        #[property(get                   )]  };
        let attr_getp = quote! {       #[property(get="path"            )]  };
        let attr_set = quote! {        #[property(            set       )]  };
        let attr_setp = quote! {       #[property(            set="path")]  };
        let attr_get_set = quote! {    #[property(get,        set       )]  };
        let attr_get_setp = quote! {   #[property(get,        set="path")]  };
        let attr_getp_set = quote! {   #[property(get="path", set       )]  };
        let attr_getp_setp = quote! {  #[property(get="path", set="path")]  };

        // See documentation of Property<T> for this table
        // Columns: #[property] attributes | i32 style fields | Property<i32> style fields
        let combinations = [
            (attr_none, true, false),
            (attr_get, true, false),
            (attr_getp, true, true),
            (attr_set, true, false),
            (attr_setp, true, true),
            (attr_get_set, true, false),
            (attr_get_setp, true, false),
            (attr_getp_set, true, false),
            (attr_getp_setp, true, true),
        ];

        for (attr, allowed_bare, allowed_property) in &combinations {
            check_property_combination(attr, quote! { i32 }, *allowed_bare);
            check_property_combination(attr, quote! { Property<i32> }, *allowed_property);
        }
    }

    /// Tests whether a certain combination of a `#[property]` attribute (attr) and a field type
    /// (bare i32 or Property<i32>) should compile successfully
    fn check_property_combination(
        attr: &TokenStream2,
        field_type: TokenStream2,
        should_succeed: bool,
    ) {
        // Lazy because of formatting in error message
        let input = || {
            quote! {
                #[inherit(Node)]
                struct Foo {
                    #attr
                    field: #field_type
                }
            }
        };

        let derive_input: DeriveInput = syn::parse2(input()).unwrap();
        let derived = derive_native_class(&derive_input);

        if should_succeed {
            assert!(
                derived.is_ok(),
                "Valid derive expression fails to compile:\n{}",
                input().to_string()
            );
        } else {
            assert_eq!(
                derived.unwrap_err().to_string(),
                "The `#[property]` attribute requires explicit paths for `get` and `set` argument; \
                the defaults #[property], #[property(get)] and #[property(set)] are not allowed.",
                "Invalid derive expression compiles by mistake:\n{}", input().to_string()
            );
        }
    }
}

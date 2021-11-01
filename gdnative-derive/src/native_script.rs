use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Meta, MetaList, NestedMeta, Path, Stmt, Type};

mod property_args;
use property_args::{PropertyAttrArgs, PropertyAttrArgsBuilder};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) base: Type,
    pub(crate) register_callback: Option<Path>,
    pub(crate) user_data: Type,
    pub(crate) properties: Vec<(Ident, PropertyAttrArgs)>,
    pub(crate) no_constructor: bool,
}

pub(crate) fn impl_empty_nativeclass(derive_input: &DeriveInput) -> TokenStream2 {
    let derived = crate::automatically_derived();
    let name = &derive_input.ident;

    quote! {
        #derived
        impl ::gdnative::nativescript::NativeClass for #name {
            type Base = ::gdnative::api::Object;
            type UserData = ::gdnative::nativescript::user_data::LocalCellData<Self>;

            fn class_name() -> &'static str {
                unimplemented!()
            }
            fn init(owner: ::gdnative::object::TRef<'_, Self::Base, Shared>) -> Self {
                unimplemented!()
            }
        }
    }
}

pub(crate) fn derive_native_class(derive_input: &DeriveInput) -> Result<TokenStream, syn::Error> {
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
        let properties = data.properties.into_iter().map(|(ident, config)| {
            let with_default = config
                .default
                .map(|default_value| quote!(.with_default(#default_value)));
            let with_hint = config.hint.map(|hint_fn| quote!(.with_hint(#hint_fn())));

            let with_usage = if config.no_editor {
                Some(quote!(.with_usage(::gdnative::nativescript::export::property::Usage::NOEDITOR)))
            } else {
                None
            };

            let before_get: Option<Stmt> = config
                .before_get
                .map(|path_expr| parse_quote!(#path_expr(this, _owner);));

            let after_get: Option<Stmt> = config
                .after_get
                .map(|path_expr| parse_quote!(#path_expr(this, _owner);));

            let before_set: Option<Stmt> = config
                .before_set
                .map(|path_expr| parse_quote!(#path_expr(this, _owner);));

            let after_set: Option<Stmt> = config
                .after_set
                .map(|path_expr| parse_quote!(#path_expr(this, _owner);));

            let label = config.path.unwrap_or_else(|| format!("{}", ident));
            quote!({
                builder.add_property(#label)
                    #with_default
                    #with_hint
                    #with_usage
                    .with_ref_getter(|this: &#name, _owner: ::gdnative::object::TRef<Self::Base>| {
                        #before_get
                        let res = &this.#ident;
                        #after_get
                        res
                    })
                    .with_setter(|this: &mut #name, _owner: ::gdnative::object::TRef<Self::Base>, v| {
                        #before_set
                        this.#ident = v;
                        #after_set
                    })
                    .done();
            })
        });

        // string variant needed for the `class_name` function.
        let name_str = quote!(#name).to_string();

        let init = if data.no_constructor {
            None
        } else {
            Some(quote! {
                fn init(owner: ::gdnative::object::TRef<Self::Base>) -> Self {
                    Self::new(::gdnative::nativescript::OwnerArg::from_safe_ref(owner))
                }
            })
        };

        quote!(
            #derived
            impl ::gdnative::nativescript::NativeClass for #name {
                type Base = #base;
                type UserData = #user_data;

                fn class_name() -> &'static str {
                    #name_str
                }

                #init

                fn register_properties(builder: &::gdnative::nativescript::export::ClassBuilder<Self>) {
                    #(#properties)*;
                    #register_callback
                }
            }
        )
    };

    // create output token stream
    Ok(trait_impl.into())
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
                quote! { ::gdnative::nativescript::user_data::DefaultUserData<#ident> },
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
                        let attr_args_builder =
                            property_args.get_or_insert_with(PropertyAttrArgsBuilder::default);

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
                        property_args.get_or_insert_with(PropertyAttrArgsBuilder::default);
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
}

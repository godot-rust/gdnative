use proc_macro::TokenStream;
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Meta, MetaList, NestedMeta, Path, Stmt, Type};

mod property_args;
use property_args::{PropertyAttrArgs, PropertyAttrArgsBuilder};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) base: Type,
    pub(crate) register_callback: Option<Path>,
    pub(crate) user_data: Type,
    pub(crate) properties: HashMap<Ident, PropertyAttrArgs>,
    pub(crate) no_constructor: bool,
}

pub(crate) fn derive_native_class(input: TokenStream) -> TokenStream {
    let data = match parse_derive_input(input) {
        Ok(val) => val,
        Err(err) => return err,
    };

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
            let with_default = if let Some(default_value) = &config.default {
                Some(quote!(.with_default(#default_value)))
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
                    .with_ref_getter(|this: &#name, _owner: ::gdnative::TRef<Self::Base>| {
                        #before_get
                        let res = &this.#ident;
                        #after_get
                        res
                    })
                    .with_setter(|this: &mut #name, _owner: ::gdnative::TRef<Self::Base>, v| {
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
                fn init(owner: ::gdnative::TRef<Self::Base>) -> Self {
                    Self::new(::gdnative::nativescript::OwnerArg::from_safe_ref(owner))
                }
            })
        };

        quote!(
            impl ::gdnative::nativescript::NativeClass for #name {
                type Base = #base;
                type UserData = #user_data;

                fn class_name() -> &'static str {
                    #name_str
                }

                #init

                fn register_properties(builder: &::gdnative::nativescript::init::ClassBuilder<Self>) {
                    #(#properties)*;
                    #register_callback
                }
            }
        )
    };

    // create output token stream
    trait_impl.into()
}

fn parse_derive_input(input: TokenStream) -> Result<DeriveData, TokenStream> {
    let span = proc_macro2::Span::call_site();

    let input = match syn::parse_macro_input::parse::<DeriveInput>(input) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_compile_error().into());
        }
    };

    let ident = input.ident;

    let inherit_attr = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("inherit"))
        .ok_or_else(|| {
            syn::Error::new(span, "No \"inherit\" attribute found").to_compile_error()
        })?;

    // read base class
    let base = inherit_attr
        .parse_args::<Type>()
        .map_err(|err| err.to_compile_error())?;

    let register_callback = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("register_with"))
        .map(|attr| attr.parse_args::<Path>().map_err(|e| e.to_compile_error()))
        .transpose()?;

    let user_data = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("user_data"))
        .map(|attr| {
            attr.parse_args::<Type>()
                .map_err(|err| err.to_compile_error())
        })
        .unwrap_or_else(|| {
            Ok(syn::parse::<Type>(
                quote! { ::gdnative::nativescript::user_data::DefaultUserData<#ident> }.into(),
            )
            .expect("quoted tokens for default userdata should be a valid type"))
        })?;

    let no_constructor = input
        .attrs
        .iter()
        .any(|a| a.path.is_ident("no_constructor"));

    // make sure it's a struct
    let struct_data = if let Data::Struct(data) = input.data {
        data
    } else {
        return Err(
            syn::Error::new(span, "NativeClass derive macro only works on structs.")
                .to_compile_error()
                .into(),
        );
    };

    // Find all fields with a `#[property]` attribute
    let mut properties = HashMap::new();

    if let Fields::Named(names) = &struct_data.fields {
        for field in &names.named {
            let mut property_args = None;

            for attr in field.attrs.iter() {
                if !attr.path.is_ident("property") {
                    continue;
                }

                let meta = attr.parse_meta().map_err(|e| e.to_compile_error())?;

                match meta {
                    Meta::List(MetaList { nested, .. }) => {
                        let attr_args_builder =
                            property_args.get_or_insert_with(PropertyAttrArgsBuilder::default);

                        for arg in &nested {
                            if let NestedMeta::Meta(Meta::NameValue(ref pair)) = arg {
                                attr_args_builder.extend(std::iter::once(pair));
                            } else {
                                let msg = format!("Unexpected argument: {:?}", arg);
                                return Err(syn::Error::new(arg.span(), msg)
                                    .to_compile_error()
                                    .into());
                            }
                        }
                    }
                    Meta::Path(_) => {
                        property_args.get_or_insert_with(PropertyAttrArgsBuilder::default);
                    }
                    m => {
                        let msg = format!("Unexpected meta variant: {:?}", m);
                        return Err(syn::Error::new(m.span(), msg).to_compile_error().into());
                    }
                }
            }

            if let Some(builder) = property_args {
                let ident = field.ident.clone().ok_or_else(|| {
                    syn::Error::new(field.ident.span(), "Fields should be named").to_compile_error()
                })?;
                properties.insert(ident, builder.done());
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

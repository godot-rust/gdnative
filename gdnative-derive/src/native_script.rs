use proc_macro::TokenStream;
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields, Ident, Meta, MetaList, NestedMeta, Path, Type};

mod property_args;
use property_args::{PropertyAttrArgs, PropertyAttrArgsBuilder};

pub(crate) struct DeriveData {
    pub(crate) name: Ident,
    pub(crate) base: Type,
    pub(crate) register_callback: Option<Path>,
    pub(crate) user_data: Type,
    pub(crate) properties: HashMap<Ident, PropertyAttrArgs>,
}

pub(crate) fn derive_native_class(input: TokenStream) -> TokenStream {
    let data = parse_derive_input(input);

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

            let label = config.path.unwrap_or_else(|| format!("{}", ident));
            quote!({
                builder.add_property(#label)
                    #with_default
                    .with_ref_getter(|this: &#name, _| &this.#ident)
                    .with_setter(|this: &mut #name, _, v| this.#ident = v)
                    .done();
            })
        });

        // string variant needed for the `class_name` function.
        let name_str = quote!(#name).to_string();

        quote!(
            impl gdnative::NativeClass for #name {
                type Base = #base;
                type UserData = #user_data;

                fn class_name() -> &'static str {
                    #name_str
                }

                fn init(owner: Self::Base) -> Self {
                    Self::_init(owner)
                }

                fn register_properties(builder: &gdnative::init::ClassBuilder<Self>) {
                    #(#properties)*;
                    #register_callback
                }
            }
        )
    };

    // create output token stream
    trait_impl.into()
}

fn parse_derive_input(input: TokenStream) -> DeriveData {
    let input = match syn::parse_macro_input::parse::<DeriveInput>(input) {
        Ok(val) => val,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let ident = input.ident;

    let inherit_attr = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("inherit"))
        .expect("No \"inherit\" attribute found");

    // read base class
    let base = inherit_attr
        .parse_args::<Type>()
        .expect("`inherits` attribute requires the base type as an argument.");

    let register_callback = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("register_with"))
        .map(|attr| {
            attr.parse_args::<Path>()
                .expect("`register_with` attributes requires a function as an argument.")
        });

    let user_data = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("user_data"))
        .map(|attr| {
            attr.parse_args::<Type>()
                .expect("`userdata` attribute requires a type as an argument.")
        })
        .unwrap_or_else(|| {
            syn::parse::<Type>(quote! { ::gdnative::user_data::DefaultUserData<#ident> }.into())
                .expect("quoted tokens should be a valid type")
        });

    // make sure it's a struct
    let struct_data = if let Data::Struct(data) = input.data {
        data
    } else {
        panic!("NativeClass derive macro only works on structs.");
    };

    // read exported properties
    let properties = if let Fields::Named(names) = &struct_data.fields {
        names
            .named
            .iter()
            .filter_map(|field| {
                let mut property_args = None;

                for attr in field.attrs.iter() {
                    if !attr.path.is_ident("property") {
                        continue;
                    }

                    let meta = attr
                        .parse_meta()
                        .expect("should be able to parse attribute arguments");

                    match meta {
                        Meta::List(MetaList { nested, .. }) => {
                            property_args
                                .get_or_insert_with(PropertyAttrArgsBuilder::default)
                                .extend(nested.iter().map(|arg| match arg {
                                    NestedMeta::Meta(Meta::NameValue(ref pair)) => pair,
                                    _ => panic!("unexpected argument: {:?}", arg),
                                }));
                        }
                        Meta::Path(_) => {
                            property_args.get_or_insert_with(PropertyAttrArgsBuilder::default);
                        }
                        _ => {
                            panic!("unexpected meta variant: {:?}", meta);
                        }
                    }
                }

                property_args.map(|builder| {
                    let ident = field.ident.clone().expect("fields should be named");
                    (ident, builder.done())
                })
            })
            .collect::<HashMap<_, _>>()
    } else {
        HashMap::new()
    };

    DeriveData {
        name: ident,
        base,
        register_callback,
        user_data,
        properties,
    }
}

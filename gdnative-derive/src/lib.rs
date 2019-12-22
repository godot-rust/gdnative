extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod derive_conv_variant;
mod derive_macro;
mod method_macro;

#[proc_macro_attribute]
pub fn methods(meta: TokenStream, input: TokenStream) -> TokenStream {
    let (impl_block, export) = method_macro::parse_method_export(meta, input);

    let output = {
        let class_name = export.class_ty;

        let methods = export
            .methods
            .into_iter()
            .map(|m| {
                let name = m.ident.clone().to_string();

                quote!(
                    {
                        let method = gdnative::godot_wrap_method!(
                            #class_name,
                            #m
                        );

                        builder.add_method(#name, method);
                    }
                )
            })
            .collect::<Vec<_>>();

        quote::quote!(

            #impl_block

            impl gdnative::NativeClassMethods for #class_name {

                fn register(builder: &gdnative::init::ClassBuilder<Self>) {
                    use gdnative::init::*;

                    #(#methods)*
                }

            }

        )
    };

    TokenStream::from(output)
}

#[proc_macro_derive(
    NativeClass,
    attributes(inherit, export, user_data, property, register_with)
)]
pub fn derive_native_class(input: TokenStream) -> TokenStream {
    let data = derive_macro::parse_derive_input(input.clone());

    // generate NativeClass impl
    let trait_impl = {
        let name = data.name;
        let base = data.base;
        let user_data = data.user_data;
        let register_callback = data
            .register_callback
            .map(|function_path| quote!(#function_path(builder);))
            .unwrap_or(quote!({}));
        let properties = data.properties.iter().map(|(ident, config)| {
            let default_value = &config.default;
            let label = format!("base/{}", ident);
            quote!({
                builder.add_property(gdnative::init::Property{
                    name: #label,
                    getter: |this: &#name| this.#ident,
                    setter: |this: &mut #name, v| this.#ident = v,
                    default: #default_value,
                    usage: gdnative::init::PropertyUsage::DEFAULT,
                    hint: gdnative::init::PropertyHint::None
                });
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

#[proc_macro_derive(ToVariant)]
pub fn derive_to_variant(input: TokenStream) -> TokenStream {
    derive_conv_variant::derive_to_variant(input)
}

#[proc_macro_derive(FromVariant)]
pub fn derive_from_variant(input: TokenStream) -> TokenStream {
    derive_conv_variant::derive_from_variant(input)
}

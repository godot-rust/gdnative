extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

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

#[proc_macro_derive(NativeClass, attributes(inherit, export, user_data))]
pub fn derive_native_class(input: TokenStream) -> TokenStream {
    let data = derive_macro::parse_derive_input(input.clone());

    // generate NativeClass impl
    let trait_impl = {
        let name = data.name;
        let base = data.base;
        let user_data = data.user_data;

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
            }
        )
    };

    // create output token stream
    trait_impl.into()
}

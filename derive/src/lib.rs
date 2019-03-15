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

use syn::{FnArg, ImplItem, ItemImpl, Pat, PatIdent, Signature, Type};

use proc_macro::TokenStream;
use std::boxed::Box;
use syn::export::Span;

pub(crate) struct ClassMethodExport {
    pub(crate) class_ty: Box<Type>,
    pub(crate) methods: Vec<ExportMethod>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct ExportMethod {
    pub(crate) sig: Signature,
    pub(crate) args: ExportArgs,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct ExportArgs {
    pub(crate) optional_args: Option<usize>,
}

pub(crate) fn derive_methods(meta: TokenStream, input: TokenStream) -> TokenStream {
    let (impl_block, export) = parse_method_export(meta, input);

    let output = {
        let class_name = export.class_ty;

        let methods = export
            .methods
            .into_iter()
            .map(|ExportMethod { sig, args }| {
                let name = sig.ident;
                let name_string = name.to_string();
                let ret_ty = match sig.output {
                    syn::ReturnType::Default => quote!(()),
                    syn::ReturnType::Type(_, ty) => quote!( #ty ),
                };

                let arg_count = sig.inputs.len();

                if arg_count < 2 {
                    panic!("exported methods must take self and owner as arguments.");
                }

                let optional_args = match args.optional_args {
                    Some(count) => {
                        let max_optional = arg_count - 2; // self and owner
                        if count > max_optional {
                            panic!(
                                "there can be at most {} optional arguments, got {}",
                                max_optional, count
                            );
                        }
                        count
                    }
                    None => 0,
                };

                let args = sig.inputs.iter().enumerate().map(|(n, arg)| {
                    if n < arg_count - optional_args {
                        quote!(#arg ,)
                    } else {
                        quote!(#[opt] #arg ,)
                    }
                });

                quote!(
                    {
                        let method = gdnative::godot_wrap_method!(
                            #class_name,
                            fn #name ( #( #args )* ) -> #ret_ty
                        );

                        builder.add_method(#name_string, method);
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

/// Parse the input.
///
/// Returns the TokenStream of the impl block together with a description of methods to export.
fn parse_method_export(_meta: TokenStream, input: TokenStream) -> (ItemImpl, ClassMethodExport) {
    let ast = match syn::parse_macro_input::parse::<ItemImpl>(input) {
        Ok(impl_block) => impl_block,
        Err(err) => {
            // if the impl block is ill-formed there is no point in error handling.
            panic!("{}", err);
        }
    };

    impl_gdnative_expose(ast)
}

/// Extract the data to export from the impl block.
#[allow(clippy::single_match)]
fn impl_gdnative_expose(ast: ItemImpl) -> (ItemImpl, ClassMethodExport) {
    // the ast input is used for inspecting.
    // this clone is used to remove all attributes so that the resulting
    // impl block actually compiles again.
    let mut result = ast.clone();

    // This is done by removing all items first, they will be added back on later
    result.items.clear();

    // data used for generating the exported methods.
    let mut export = ClassMethodExport {
        class_ty: ast.self_ty,
        methods: vec![],
    };

    let mut methods_to_export: Vec<ExportMethod> = Vec::new();

    // extract all methods that have the #[export] attribute.
    // add all items back to the impl block again.
    for func in ast.items {
        let item = match func {
            ImplItem::Method(mut method) => {
                let mut export_args = None;

                // only allow the "outer" style, aka #[thing] item.
                method.attrs.retain(|attr| {
                    let correct_style = match attr.style {
                        syn::AttrStyle::Outer => true,
                        _ => false,
                    };

                    if correct_style {
                        let last_seg = attr
                            .path
                            .segments
                            .iter()
                            .last()
                            .map(|i| i.ident.to_string());

                        if let Some("export") = last_seg.as_deref() {
                            let _export_args = export_args.get_or_insert_with(ExportArgs::default);
                            if !attr.tokens.is_empty() {
                                use quote::ToTokens;
                                use syn::{Meta, MetaNameValue, NestedMeta};

                                let meta =
                                    attr.parse_meta().expect("cannot parse attribute arguments");

                                let pairs: Vec<_> = match meta {
                                    Meta::List(list) => list
                                        .nested
                                        .into_pairs()
                                        .map(|p| match p.into_value() {
                                            NestedMeta::Meta(Meta::NameValue(pair)) => pair,
                                            unexpected => panic!(
                                                "unexpected argument in list: {}",
                                                unexpected.into_token_stream()
                                            ),
                                        })
                                        .collect(),
                                    Meta::NameValue(pair) => vec![pair],
                                    meta => panic!(
                                        "unexpected attribute argument: {}",
                                        meta.into_token_stream()
                                    ),
                                };

                                for MetaNameValue { path, .. } in pairs.into_iter() {
                                    let last =
                                        path.segments.last().expect("the path should not be empty");
                                    let unexpected = last.ident.to_string();
                                    panic!("unknown option for export: `{}`", unexpected);
                                }
                            }

                            return false;
                        }
                    }

                    true
                });

                if let Some(mut export_args) = export_args.take() {
                    let mut optional_args = None;

                    for (n, arg) in method.sig.inputs.iter_mut().enumerate() {
                        let attrs = match arg {
                            FnArg::Receiver(a) => &mut a.attrs,
                            FnArg::Typed(a) => &mut a.attrs,
                        };

                        let mut is_optional = false;

                        attrs.retain(|attr| {
                            if attr.path.is_ident("opt") {
                                is_optional = true;
                                false
                            } else {
                                true
                            }
                        });

                        if is_optional {
                            if n < 2 {
                                panic!("self and owner cannot be optional");
                            }

                            *optional_args.get_or_insert(0) += 1;
                        } else if optional_args.is_some() {
                            panic!("cannot add required parameters after optional ones");
                        }
                    }

                    export_args.optional_args = optional_args;

                    methods_to_export.push(ExportMethod {
                        sig: method.sig.clone(),
                        args: export_args,
                    });
                }

                ImplItem::Method(method)
            }
            item => item,
        };

        result.items.push(item);
    }

    // check if the export methods have the proper "shape", the write them
    // into the list of things to export.
    {
        for mut method in methods_to_export {
            let generics = &method.sig.generics;

            if generics.type_params().count() > 0 {
                eprintln!("type parameters not allowed in exported functions");
                continue;
            }
            if generics.lifetimes().count() > 0 {
                eprintln!("lifetime parameters not allowed in exported functions");
                continue;
            }
            if generics.const_params().count() > 0 {
                eprintln!("const parameters not allowed in exported functions");
                continue;
            }

            // remove "mut" from arguments.
            // give every wildcard a (hopefully) unique name.
            method
                .sig
                .inputs
                .iter_mut()
                .enumerate()
                .for_each(|(i, arg)| match arg {
                    FnArg::Typed(cap) => match *cap.pat.clone() {
                        Pat::Wild(_) => {
                            let name = format!("___unused_arg_{}", i);

                            cap.pat = Box::new(Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: syn::Ident::new(&name, Span::call_site()),
                                subpat: None,
                            }));
                        }
                        Pat::Ident(mut ident) => {
                            ident.mutability = None;
                            cap.pat = Box::new(Pat::Ident(ident));
                        }
                        _ => {}
                    },
                    _ => {}
                });

            // The calling site is already in an unsafe block, so removing it from just the
            // exported binding is fine.
            method.sig.unsafety = None;

            export.methods.push(method);
        }
    }

    (result, export)
}

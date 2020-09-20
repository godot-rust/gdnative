use syn::{spanned::Spanned, FnArg, ImplItem, ItemImpl, Pat, PatIdent, Signature, Type};

use proc_macro::TokenStream;
use std::boxed::Box;

pub(crate) struct ClassMethodExport {
    pub(crate) class_ty: Box<Type>,
    pub(crate) methods: Vec<ExportMethod>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct ExportMethod {
    pub(crate) sig: Signature,
    pub(crate) args: ExportArgs,
    pub(crate) rpc: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct ExportArgs {
    pub(crate) optional_args: Option<usize>,
}

pub(crate) fn derive_methods(meta: TokenStream, input: TokenStream) -> TokenStream {
    let (impl_block, export) = match parse_method_export(meta, input) {
        Ok(val) => val,
        Err(toks) => return toks,
    };

    let output = {
        let class_name = export.class_ty;

        let builder = syn::Ident::new("builder", proc_macro2::Span::call_site());

        let methods = export
            .methods
            .into_iter()
            .map(|ExportMethod { sig, args, rpc }| {
                let sig_span = sig.ident.span();

                let name = sig.ident;
                let name_string = name.to_string();
                let ret_span = sig.output.span();
                let ret_ty = match sig.output {
                    syn::ReturnType::Default => quote_spanned!(ret_span => ()),
                    syn::ReturnType::Type(_, ty) => quote_spanned!( ret_span => #ty ),
                };

                let arg_count = sig.inputs.len();

                if arg_count < 2 {
                    return syn::Error::new(
                        sig_span,
                        "exported methods must take self and owner as arguments",
                    )
                    .to_compile_error();
                }

                let optional_args = match args.optional_args {
                    Some(count) => {
                        let max_optional = arg_count - 2; // self and owner
                        if count > max_optional {
                            let message = format!(
                                "there can be at most {} optional arguments, got {}",
                                max_optional, count,
                            );
                            return syn::Error::new(sig_span, message).to_compile_error();
                        }
                        count
                    }
                    None => 0,
                };

                let args = sig.inputs.iter().enumerate().map(|(n, arg)| {
                    let span = arg.span();
                    if n < arg_count - optional_args {
                        quote_spanned!(span => #arg ,)
                    } else {
                        quote_spanned!(span => #[opt] #arg ,)
                    }
                });

                quote_spanned!( sig_span=>
                    {
                        let method = ::gdnative::godot_wrap_method!(
                            #class_name,
                            fn #name ( #( #args )* ) -> #ret_ty
                        );

                        #builder.add_method(#name_string, method, #rpc);
                    }
                )
            })
            .collect::<Vec<_>>();

        quote::quote!(

            #impl_block

            impl gdnative::nativescript::NativeClassMethods for #class_name {

                fn register(#builder: &::gdnative::nativescript::init::ClassBuilder<Self>) {
                    use gdnative::nativescript::init::*;

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
fn parse_method_export(
    _meta: TokenStream,
    input: TokenStream,
) -> Result<(ItemImpl, ClassMethodExport), TokenStream> {
    let ast = match syn::parse_macro_input::parse::<ItemImpl>(input) {
        Ok(impl_block) => impl_block,
        Err(err) => {
            return Err(err.to_compile_error().into());
        }
    };

    Ok(impl_gdnative_expose(ast))
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
        let items = match func {
            ImplItem::Method(mut method) => {
                let mut export_args = None;
                let mut rpc = "disabled";

                let mut errors = vec![];

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

                                let meta = match attr.parse_meta() {
                                    Ok(val) => val,
                                    Err(err) => {
                                        errors.push(err);
                                        return false;
                                    }
                                };

                                let pairs: Vec<_> = match meta {
                                    Meta::List(list) => list
                                        .nested
                                        .into_pairs()
                                        .filter_map(|p| {
                                            let span = p.span();
                                            match p.into_value() {
                                                NestedMeta::Meta(Meta::NameValue(pair)) => {
                                                    Some(pair)
                                                }
                                                unexpected => {
                                                    let msg = format!(
                                                        "unexpected argument in list: {}",
                                                        unexpected.into_token_stream()
                                                    );
                                                    errors.push(syn::Error::new(span, msg));
                                                    None
                                                }
                                            }
                                        })
                                        .collect(),
                                    Meta::NameValue(pair) => vec![pair],
                                    meta => {
                                        let span = meta.span();
                                        let msg = format!(
                                            "unexpected attribute argument: {}",
                                            meta.into_token_stream()
                                        );
                                        errors.push(syn::Error::new(span, msg));
                                        return false;
                                    }
                                };

                                for MetaNameValue {
                                    path,
                                    eq_token: _,
                                    lit,
                                } in pairs
                                {
                                    let last = match path.segments.last() {
                                        Some(val) => val,
                                        None => {
                                            errors.push(syn::Error::new(
                                                path.span(),
                                                "the path should not be empty",
                                            ));
                                            return false;
                                        }
                                    };
                                    let path = last.ident.to_string();

                                    // Match rpc mode
                                    match path.as_str() {
                                        "rpc" => {
                                            let value = if let syn::Lit::Str(lit_str) = lit {
                                                lit_str.value()
                                            } else {
                                                errors.push(syn::Error::new(
                                                    last.span(),
                                                    "unexpected type for rpc value, expected Str",
                                                ));
                                                return false;
                                            };

                                            match value.as_str() {
                                                "remote" => {
                                                    rpc = "remote";
                                                    return false;
                                                }
                                                "remotesync" => {
                                                    rpc = "remotesync";
                                                    return false;
                                                }
                                                "master" => {
                                                    rpc = "master";
                                                    return false;
                                                }
                                                "puppet" => {
                                                    rpc = "puppet";
                                                    return false;
                                                }
                                                "disabled" => {
                                                    rpc = "disabled";
                                                    return false;
                                                }
                                                _ => {
                                                    errors.push(syn::Error::new(
                                                        last.span(),
                                                        format!(
                                                            "unexpected value for rpc: {}",
                                                            value
                                                        ),
                                                    ));
                                                    return false;
                                                }
                                            }
                                        }
                                        _ => (),
                                    }

                                    let msg = format!("unknown option for export: `{}`", path);
                                    errors.push(syn::Error::new(last.span(), msg));
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
                                errors.push(syn::Error::new(
                                    arg.span(),
                                    "self or owner cannot be optional",
                                ));
                                continue;
                            }

                            *optional_args.get_or_insert(0) += 1;
                        } else if optional_args.is_some() {
                            errors.push(syn::Error::new(
                                arg.span(),
                                "cannot add required parameters after optional ones",
                            ));
                            continue;
                        }
                    }

                    export_args.optional_args = optional_args;

                    methods_to_export.push(ExportMethod {
                        sig: method.sig.clone(),
                        args: export_args,
                        rpc: rpc.to_string(),
                    });
                }

                errors
                    .into_iter()
                    .map(|err| ImplItem::Verbatim(err.to_compile_error()))
                    .chain(std::iter::once(ImplItem::Method(method)))
                    .collect()
            }
            item => vec![item],
        };

        result.items.extend(items);
    }

    // check if the export methods have the proper "shape", the write them
    // into the list of things to export.
    {
        for mut method in methods_to_export {
            let generics = &method.sig.generics;
            let span = method.sig.ident.span();

            if generics.type_params().count() > 0 {
                let toks =
                    syn::Error::new(span, "Type parameters not allowed in exported functions")
                        .to_compile_error();
                result.items.push(ImplItem::Verbatim(toks));
                continue;
            }
            if generics.lifetimes().count() > 0 {
                let toks = syn::Error::new(
                    span,
                    "Lifetime parameters not allowed in exported functions",
                )
                .to_compile_error();
                result.items.push(ImplItem::Verbatim(toks));
                continue;
            }
            if generics.const_params().count() > 0 {
                let toks =
                    syn::Error::new(span, "const parameters not allowed in exported functions")
                        .to_compile_error();
                result.items.push(ImplItem::Verbatim(toks));
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
                                ident: syn::Ident::new(&name, span),
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

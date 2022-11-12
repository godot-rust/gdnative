use syn::{spanned::Spanned, FnArg, ImplItem, ItemImpl, Pat, PatIdent, Signature, Type};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use std::boxed::Box;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum RpcMode {
    Disabled,
    Remote,
    RemoteSync,
    Master,
    Puppet,
    MasterSync,
    PuppetSync,
}

impl RpcMode {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "remote" => Some(RpcMode::Remote),
            "remote_sync" => Some(RpcMode::RemoteSync),
            "master" => Some(RpcMode::Master),
            "puppet" => Some(RpcMode::Puppet),
            "disabled" => Some(RpcMode::Disabled),
            "master_sync" => Some(RpcMode::MasterSync),
            "puppet_sync" => Some(RpcMode::PuppetSync),
            _ => None,
        }
    }
}

impl Default for RpcMode {
    fn default() -> Self {
        RpcMode::Disabled
    }
}

impl ToTokens for RpcMode {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            RpcMode::Disabled => tokens.extend(quote!(RpcMode::Disabled)),
            RpcMode::Remote => tokens.extend(quote!(RpcMode::Remote)),
            RpcMode::RemoteSync => tokens.extend(quote!(RpcMode::RemoteSync)),
            RpcMode::Master => tokens.extend(quote!(RpcMode::Master)),
            RpcMode::Puppet => tokens.extend(quote!(RpcMode::Puppet)),
            RpcMode::MasterSync => tokens.extend(quote!(RpcMode::MasterSync)),
            RpcMode::PuppetSync => tokens.extend(quote!(RpcMode::PuppetSync)),
        }
    }
}

pub(crate) struct ClassMethodExport {
    pub(crate) class_ty: Box<Type>,
    pub(crate) methods: Vec<ExportMethod>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct ExportMethod {
    pub(crate) sig: Signature,
    pub(crate) export_args: ExportArgs,
    pub(crate) optional_args: Option<usize>,
    pub(crate) exist_base_arg: bool,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct ExportArgs {
    pub(crate) is_old_syntax: bool,
    pub(crate) rpc_mode: Option<RpcMode>,
    pub(crate) name_override: Option<String>,
    pub(crate) is_deref_return: bool,
}

pub(crate) fn derive_methods(item_impl: ItemImpl) -> TokenStream2 {
    let derived = crate::automatically_derived();
    let (impl_block, export) = impl_gdnative_expose(item_impl);

    let class_name = export.class_ty;

    let builder = syn::Ident::new("builder", proc_macro2::Span::call_site());

    let methods = export
        .methods
        .into_iter()
        .map(|export_method| {
            let ExportMethod {
                sig,
                export_args,
                exist_base_arg,
                ..
            } = &export_method;

            let sig_span = sig.ident.span();

            let name = sig.ident.clone();
            let name_string = export_args
                .name_override
                .clone()
                .unwrap_or_else(|| name.to_string());
            let ret_span = sig.output.span();

            let arg_count = sig.inputs.len();

            if arg_count == 0 {
                return syn::Error::new(
                    sig_span,
                    "#[method] exported methods must take self parameter",
                )
                .to_compile_error();
            }

            if export_args.is_old_syntax && !exist_base_arg {
                return syn::Error::new(
                    sig_span,
                    "deprecated #[export] methods must take second parameter (base/owner)",
                )
                .to_compile_error();
            }

            let rpc = export_args.rpc_mode.unwrap_or(RpcMode::Disabled);
            let is_deref_return = export_args.is_deref_return;

            let warn_deprecated_export = if export_args.is_old_syntax {
                let warning = crate::emit_warning(
                    sig_span,
                    "deprecated_export_syntax",
                    concat!(
                        "\n",
                        "#[export] is deprecated and will be removed in a future godot-rust version. Use #[method] instead.\n\n",
                        "For more information, see https://godot-rust.github.io/docs/gdnative/derive/derive.NativeClass.html."
                    )
                );

                Some(quote_spanned!(sig_span=>#warning;))
            } else {
                None
            };

            // See gdnative-core::export::deprecated_reference_return!()
            let warn_deprecated_ref_return = if let syn::ReturnType::Type(_, ty) = &sig.output {
                if !is_deref_return && matches!(**ty, syn::Type::Reference(_)) {
                    let warning = crate::emit_warning(
                        ret_span,
                        "deprecated_reference_return",
                        "This function does not actually pass by reference to the Godot engine. You can clarify by writing #[method(deref_return)]."
                    );

                    quote_spanned!(ret_span=>#warning;)
                } else {
                    quote_spanned!(ret_span=>)
                }
            } else {
                quote_spanned!(ret_span=>)
            };

            let method = wrap_method(&class_name, &export_method)
                .unwrap_or_else(|err| err.to_compile_error());

            quote_spanned!( sig_span=>
                {
                    #builder.method(#name_string, #method)
                        .with_rpc_mode(#rpc)
                        .done_stateless();

                    #warn_deprecated_export
                    #warn_deprecated_ref_return
                }
            )
        })
        .collect::<Vec<_>>();

    quote::quote!(
        #impl_block

        #derived
        impl gdnative::export::NativeClassMethods for #class_name {
            fn nativeclass_register(#builder: &::gdnative::export::ClassBuilder<Self>) {
                use gdnative::export::*;

                #(#methods)*
            }
        }

    )
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

    // extract all methods that have the #[method] attribute
    // add all items back to the impl block again.
    for func in ast.items {
        let items = match func {
            ImplItem::Method(mut method) => {
                let mut export_args = None;
                let mut errors = vec![];

                // only allow the "outer" style, aka #[thing] item.
                method.attrs.retain(|attr| {
                    if matches!(attr.style, syn::AttrStyle::Outer) {
                        let last_seg = attr
                            .path
                            .segments
                            .iter()
                            .last()
                            .map(|i| i.ident.to_string());

                        let (is_export, is_old_syntax, macro_name) =
                            if let Some("export") = last_seg.as_deref() {
                                (true, true, "export")
                            } else if let Some("method") = last_seg.as_deref() {
                                (true, false, "method")
                            } else {
                                (false, false, "unknown")
                            };

                        if is_export {
                            use syn::{punctuated::Punctuated, Lit, Meta, NestedMeta};
                            let mut export_args =
                                export_args.get_or_insert_with(ExportArgs::default);
                            export_args.is_old_syntax = is_old_syntax;

                            // Codes like #[macro(path, name = "value")] are accepted.
                            // Codes like #[path], #[name = "value"] or #[macro("lit")] are not accepted.
                            let nested_meta_iter = match attr.parse_meta() {
                                Err(err) => {
                                    errors.push(err);
                                    return false;
                                }
                                Ok(Meta::NameValue(name_value)) => {
                                    let span = name_value.span();
                                    let msg = "NameValue syntax is not valid";
                                    errors.push(syn::Error::new(span, msg));
                                    return false;
                                }
                                Ok(Meta::Path(_)) => {
                                    Punctuated::<NestedMeta, syn::token::Comma>::new().into_iter()
                                }
                                Ok(Meta::List(list)) => list.nested.into_iter(),
                            };
                            for nested_meta in nested_meta_iter {
                                let (path, lit) = match &nested_meta {
                                    NestedMeta::Lit(param) => {
                                        let span = param.span();
                                        let msg = "Literal item is not valid";
                                        errors.push(syn::Error::new(span, msg));
                                        continue;
                                    }
                                    NestedMeta::Meta(param) => match param {
                                        Meta::List(list) => {
                                            let span = list.span();
                                            let msg = "List item is not valid";
                                            errors.push(syn::Error::new(span, msg));
                                            continue;
                                        }
                                        Meta::Path(path) => (path, None),
                                        Meta::NameValue(name_value) => {
                                            (&name_value.path, Some(&name_value.lit))
                                        }
                                    },
                                };
                                if path.is_ident("rpc") {
                                    // rpc mode
                                    match lit {
                                        None => {
                                            errors.push(syn::Error::new(
                                                nested_meta.span(),
                                                "`rpc` parameter requires string value",
                                            ));
                                        }
                                        Some(Lit::Str(str)) => {
                                            let value = str.value();
                                            if let Some(mode) = RpcMode::parse(value.as_str()) {
                                                if export_args.rpc_mode.replace(mode).is_some() {
                                                    errors.push(syn::Error::new(
                                                        nested_meta.span(),
                                                        "`rpc` mode was set more than once",
                                                    ));
                                                }
                                            } else {
                                                errors.push(syn::Error::new(
                                                    nested_meta.span(),
                                                    format!(
                                                        "unexpected value for `rpc`: {}",
                                                        value
                                                    ),
                                                ));
                                            }
                                        }
                                        _ => {
                                            errors.push(syn::Error::new(
                                                nested_meta.span(),
                                                "unexpected type for `rpc` value, expected string",
                                            ));
                                        }
                                    }
                                } else if path.is_ident("name") {
                                    // name override
                                    match lit {
                                        None => {
                                            errors.push(syn::Error::new(
                                                nested_meta.span(),
                                                "`name` parameter requires string value",
                                            ));
                                        }
                                        Some(Lit::Str(str)) => {
                                            if export_args
                                                .name_override
                                                .replace(str.value())
                                                .is_some()
                                            {
                                                errors.push(syn::Error::new(
                                                    nested_meta.span(),
                                                    "`name` was set more than once",
                                                ));
                                            }
                                        }
                                        _ => {
                                            errors.push(syn::Error::new(
                                                nested_meta.span(),
                                                "unexpected type for `name` value, expected string",
                                            ));
                                        }
                                    }
                                } else if path.is_ident("deref_return") {
                                    // deref return value
                                    if lit.is_some() {
                                        errors.push(syn::Error::new(
                                            nested_meta.span(),
                                            "`deref_return` does not take any values",
                                        ));
                                    } else if export_args.is_deref_return {
                                        errors.push(syn::Error::new(
                                            nested_meta.span(),
                                            "`deref_return` was set more than once",
                                        ));
                                    } else {
                                        export_args.is_deref_return = true;
                                    }
                                } else {
                                    let msg = format!(
                                        "unknown option for #[{}]: `{}`",
                                        macro_name,
                                        path.to_token_stream()
                                    );
                                    errors.push(syn::Error::new(nested_meta.span(), msg));
                                }
                            }
                            return false;
                        }
                    }

                    true
                });

                if let Some(export_args) = export_args.take() {
                    let (optional_args, exist_base_arg) =
                        parse_signature_attrs(&mut method.sig, &export_args, &mut errors);

                    methods_to_export.push(ExportMethod {
                        sig: method.sig.clone(),
                        export_args,
                        optional_args,
                        exist_base_arg,
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

            // remove "mut" from parameters.
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

fn parse_signature_attrs(
    sig: &mut syn::Signature,
    export_args: &ExportArgs,
    errors: &mut Vec<syn::Error>,
) -> (Option<usize>, bool) {
    let mut optional_args = None;
    let mut exist_base_arg = false;

    for (n, arg) in sig.inputs.iter_mut().enumerate() {
        let attrs = match arg {
            FnArg::Receiver(a) => &mut a.attrs,
            FnArg::Typed(a) => &mut a.attrs,
        };

        let mut is_optional = false;
        let mut is_base = false;

        attrs.retain(|attr| {
            if attr.path.is_ident("opt") {
                is_optional = true;
                false
            } else if attr.path.is_ident("base") {
                is_base = true;
                false
            } else {
                true
            }
        });

        // In the old syntax, the second parameter is always the base parameter.
        if export_args.is_old_syntax && n == 1 {
            is_base = true;
        }

        if is_optional {
            if n < 2 {
                errors.push(syn::Error::new(
                    arg.span(),
                    "self or base cannot be optional",
                ));
            } else {
                *optional_args.get_or_insert(0) += 1;
            }
        } else if optional_args.is_some() {
            errors.push(syn::Error::new(
                arg.span(),
                "cannot add required parameters after optional ones",
            ));
        }

        if is_base {
            exist_base_arg = true;
            if n != 1 {
                errors.push(syn::Error::new(
                    arg.span(),
                    "base must be the second parameter.",
                ));
            }
        }
    }

    (optional_args, exist_base_arg)
}

pub(crate) fn expand_godot_wrap_method(
    input: TokenStream2,
) -> Result<TokenStream2, Vec<syn::Error>> {
    struct Input {
        class_name: syn::Type,
        is_deref_return: syn::LitBool,
        signature: syn::Signature,
    }

    impl syn::parse::Parse for Input {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let class_name = input.parse()?;
            input.parse::<Token![,]>()?;

            let is_deref_return = input.parse()?;
            input.parse::<Token![,]>()?;

            let signature = input.parse()?;

            input.parse::<Token![,]>().ok();

            if input.is_empty() {
                Ok(Input {
                    class_name,
                    is_deref_return,
                    signature,
                })
            } else {
                Err(syn::Error::new(input.span(), "expecting end of input"))
            }
        }
    }

    let Input {
        class_name,
        is_deref_return,
        mut signature,
    } = syn::parse2(input).map_err(|e| vec![e])?;

    let mut errors = Vec::new();

    let export_args = ExportArgs {
        is_old_syntax: false,
        rpc_mode: None,
        name_override: None,
        is_deref_return: is_deref_return.value,
    };

    let (optional_args, exist_base_arg) =
        parse_signature_attrs(&mut signature, &export_args, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    let export_method = ExportMethod {
        sig: signature,
        export_args,
        optional_args,
        exist_base_arg,
    };

    wrap_method(&class_name, &export_method).map_err(|e| vec![e])
}

fn wrap_method(
    class_name: &Type,
    export_method: &ExportMethod,
) -> Result<TokenStream2, syn::Error> {
    let ExportMethod {
        sig,
        export_args,
        exist_base_arg,
        optional_args,
    } = &export_method;

    let gdnative_core = crate::crate_gdnative_core();
    let automatically_derived = crate::automatically_derived();

    let sig_span = sig.ident.span();
    let ret_span = sig.output.span();

    let method_name = &sig.ident;

    let maybe_owner_arg = if *exist_base_arg {
        Some(quote_spanned! { sig_span => OwnerArg::from_safe_ref(__base), })
    } else {
        None
    };

    let normal_args_start = if *exist_base_arg { 2 } else { 1 };

    let required_args = sig.inputs.len() - normal_args_start - optional_args.unwrap_or(0);

    let declare_arg_list = sig
        .inputs
        .iter()
        .skip(normal_args_start)
        .enumerate()
        .filter_map(|(n, arg)| match arg {
            FnArg::Typed(_) => {
                let span = arg.span();
                let required = n < required_args;
                let maybe_opt = if required {
                    None
                } else {
                    Some(quote_spanned!(span => #[opt]))
                };
                Some(quote_spanned!(span => #maybe_opt #arg))
            }
            FnArg::Receiver(_) => None,
        })
        .collect::<Vec<_>>();

    let invoke_arg_list = sig
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(n, arg)| match arg {
            FnArg::Typed(pat) => {
                if n > 1 || !*exist_base_arg {
                    Some(&pat.pat)
                } else {
                    None
                }
            }
            FnArg::Receiver(_) => None,
        })
        .collect::<Vec<_>>();

    let recover = if export_args.is_deref_return {
        quote_spanned! { ret_span => std::ops::Deref::deref(&ret) }
    } else {
        quote_spanned! { ret_span => ret }
    };

    let receiver = sig
        .inputs
        .first()
        .and_then(|pat| {
            if let FnArg::Receiver(r) = pat {
                Some(r)
            } else {
                None
            }
        })
        .ok_or_else(|| syn::Error::new(sig_span, "exported method must declare a receiver"))?;

    let map_method = if receiver.reference.is_some() {
        if receiver.mutability.is_some() {
            syn::Ident::new("map_mut", sig_span)
        } else {
            syn::Ident::new("map", sig_span)
        }
    } else {
        syn::Ident::new("map_owned", sig_span)
    };

    let output = quote_spanned! { sig_span =>
        {
            #[derive(Copy, Clone, Default)]
            struct ThisMethod;

            use #gdnative_core::export::{NativeClass, OwnerArg};
            use #gdnative_core::object::{Instance, TInstance};
            use #gdnative_core::derive::FromVarargs;

            #[derive(FromVarargs)]
            #automatically_derived
            struct Args {
                #(#declare_arg_list,)*
            }

            #automatically_derived
            impl #gdnative_core::export::StaticArgsMethod<#class_name> for ThisMethod {
                type Args = Args;
                fn call(
                    &self,
                    this: TInstance<'_, #class_name, #gdnative_core::object::ownership::Shared>,
                    Args { #(#invoke_arg_list,)* }: Args,
                ) -> #gdnative_core::core_types::Variant {
                    this
                        .#map_method(|__rust_val, __base| {
                            #[allow(unused_unsafe)]
                            unsafe {
                                let ret = __rust_val.#method_name(
                                    #maybe_owner_arg
                                    #(#invoke_arg_list,)*
                                );
                                gdnative::core_types::OwnedToVariant::owned_to_variant(#recover)
                            }
                        })
                        .unwrap_or_else(|err| {
                            #gdnative_core::godot_error!("gdnative-core: method call failed with error: {}", err);
                            #gdnative_core::godot_error!("gdnative-core: check module level documentation on gdnative::user_data for more information");
                            #gdnative_core::core_types::Variant::nil()
                        })
                }

                fn site() -> Option<#gdnative_core::log::Site<'static>> {
                    Some(#gdnative_core::godot_site!(#class_name::#method_name))
                }
            }

            #gdnative_core::export::StaticArgs::new(ThisMethod)
        }
    };

    Ok(output)
}

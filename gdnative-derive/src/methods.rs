use syn::{
    spanned::Spanned, visit::Visit, FnArg, Generics, ImplItem, ItemImpl, Meta, NestedMeta, Pat,
    PatIdent, Signature, Type,
};

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use std::boxed::Box;

use crate::syntax::rpc_mode::RpcMode;
use crate::utils::find_non_concrete;

use self::mixin_args::{MixinArgsBuilder, MixinKind};

mod mixin_args;

pub(crate) struct ClassMethodExport {
    pub(crate) class_ty: Box<Type>,
    pub(crate) methods: Vec<ExportMethod>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct ExportMethod {
    /// Signature of the method *with argument attributes stripped*
    pub(crate) sig: Signature,
    pub(crate) export_args: ExportArgs,
    pub(crate) arg_kind: Vec<ArgKind>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum ArgKind {
    /// Variations of `self`
    Receiver,
    /// `#[base]`
    Base,
    /// `#[async_ctx]`
    AsyncCtx,
    /// Regular arguments
    Regular {
        /// `#[opt]`
        optional: bool,
    },
}

impl std::fmt::Display for ArgKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver => write!(f, "method receiver"),
            Self::Base => write!(f, "base/owner object"),
            Self::AsyncCtx => write!(f, "async context"),
            Self::Regular { optional: true } => write!(f, "optional argument"),
            Self::Regular { optional: false } => write!(f, "regular argument"),
        }
    }
}

impl ArgKind {
    fn strip_parse(arg: &mut FnArg, errors: &mut Vec<syn::Error>) -> (bool, Self) {
        let (receiver, attrs) = match arg {
            FnArg::Receiver(a) => (Some(a.self_token.span), &mut a.attrs),
            FnArg::Typed(a) => (None, &mut a.attrs),
        };

        let mut optional = None;
        let mut base = None;
        let mut async_ctx = None;

        let mut fail = false;

        attrs.retain(|attr| {
            if attr.path.is_ident("opt") {
                if let Some(old_span) = optional.replace(attr.path.span()) {
                    fail = true;
                    optional = Some(old_span);
                    errors.push(syn::Error::new(attr.path.span(), "duplicate attribute"));
                }
                false
            } else if attr.path.is_ident("base") {
                if let Some(old_span) = base.replace(attr.path.span()) {
                    fail = true;
                    base = Some(old_span);
                    errors.push(syn::Error::new(attr.path.span(), "duplicate attribute"));
                }
                false
            } else if attr.path.is_ident("async_ctx") {
                if let Some(old_span) = async_ctx.replace(attr.path.span()) {
                    fail = true;
                    async_ctx = Some(old_span);
                    errors.push(syn::Error::new(attr.path.span(), "duplicate attribute"));
                }
                false
            } else {
                true
            }
        });

        let mut special_kind = None;

        macro_rules! check_special_kind {
            ($ident:ident => $var:expr) => {
                if let Some($ident) = $ident {
                    if let Some(kind) = special_kind.replace($var) {
                        fail = true;
                        errors.push(syn::Error::new(
                            $ident,
                            format_args!("the {} cannot also be the {}", kind, $var),
                        ));
                        special_kind = Some(kind);
                    }
                }
            };
        }

        check_special_kind!(receiver => ArgKind::Receiver);
        check_special_kind!(base => ArgKind::Base);
        check_special_kind!(async_ctx => ArgKind::AsyncCtx);

        let kind = if let Some(special_kind) = special_kind {
            if let Some(optional) = optional {
                fail = true;
                errors.push(syn::Error::new(
                    optional,
                    format_args!(
                        "the {special_kind} cannot be optional (instead, remove the argument entirely)"
                    ),
                ));
            }

            special_kind
        } else {
            ArgKind::Regular {
                optional: optional.is_some(),
            }
        };

        (fail, kind)
    }
}

impl ExportMethod {
    fn strip_parse(
        sig: &mut Signature,
        export_args: ExportArgs,
        errors: &mut Vec<syn::Error>,
    ) -> Option<Self> {
        let mut arg_kind = Vec::new();
        let sig_span = sig.ident.span();

        let mut inputs = sig.inputs.iter_mut().enumerate();

        let mut receiver_seen = None;
        let mut base_seen = None;
        let mut async_ctx_seen = None;

        let mut fail = false;

        let is_async = export_args.is_async || sig.asyncness.is_some();

        if export_args.is_old_syntax {
            if inputs.len() < 2 {
                fail = true;
            } else {
                match inputs.next().expect("argument count checked") {
                    (n, FnArg::Receiver(_)) => {
                        arg_kind.push(ArgKind::Receiver);
                        receiver_seen = Some(n);
                    }
                    (_, arg) => {
                        errors.push(syn::Error::new(arg.span(), "expecting method receiver"));
                        fail = true;
                    }
                }

                let (n, arg) = inputs.next().expect("argument count checked");
                let (arg_fail, kind) = ArgKind::strip_parse(arg, errors);
                fail |= arg_fail;
                match kind {
                    ArgKind::Base | ArgKind::Regular { .. } => {
                        arg_kind.push(ArgKind::Base);
                        base_seen = Some(n);
                    }
                    kind => {
                        errors.push(syn::Error::new(
                            arg.span(),
                            format_args!("expecting {}, found {}", ArgKind::Base, kind),
                        ));
                        fail = true;
                    }
                };
            }

            if fail {
                errors.push(syn::Error::new(
                    sig_span,
                    "methods exported using the old syntax must declare both `self` and `owner`.",
                ));
            }
        }

        let mut regular_argument_seen = None;
        let mut optional_argument_seen = None;

        for (n, arg) in inputs {
            let (arg_fail, kind) = ArgKind::strip_parse(arg, errors);
            fail |= arg_fail;

            if let ArgKind::Regular { optional } = &kind {
                regular_argument_seen.get_or_insert(n);

                if *optional {
                    optional_argument_seen.get_or_insert(n);
                } else if let Some(idx) = optional_argument_seen {
                    fail = true;
                    errors.push(syn::Error::new(
                        arg.span(),
                        format_args!(
                            "required parameters must precede all optional ones (an optional parameter is defined at #{idx})",
                        )
                    ));
                }
            } else if let Some(idx) = regular_argument_seen {
                fail = true;
                errors.push(syn::Error::new(
                    arg.span(),
                    format_args!(
                        "special parameters must precede all regular ones (a regular parameter is defined at #{idx})",
                    )
                ));
            } else {
                let seen = match &kind {
                    ArgKind::Receiver => &mut receiver_seen,
                    ArgKind::Base => &mut base_seen,
                    ArgKind::AsyncCtx => &mut async_ctx_seen,
                    ArgKind::Regular { .. } => unreachable!(),
                };

                if let Some(idx) = seen.replace(n) {
                    *seen = Some(idx);
                    fail = true;
                    errors.push(syn::Error::new(
                        arg.span(),
                        format_args!(
                            "the special parameter {kind} must only be declared once (the same parameter is already defined at #{idx})",
                        )
                    ));
                }
            }

            if matches!(kind, ArgKind::Receiver) && !matches!(arg, FnArg::Receiver(_)) {
                fail = true;
                errors.push(syn::Error::new(
                    arg.span(),
                    "non-self receivers aren't supported yet",
                ));
            }

            if matches!(kind, ArgKind::AsyncCtx) && !is_async {
                fail = true;
                errors.push(syn::Error::new(
                    arg.span(),
                    "the async context is only available to async methods",
                ));
            }

            arg_kind.push(kind);
        }

        if fail {
            None
        } else {
            Some(ExportMethod {
                sig: sig.clone(),
                export_args,
                arg_kind,
            })
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub(crate) struct ExportArgs {
    pub(crate) is_old_syntax: bool,
    pub(crate) rpc_mode: Option<RpcMode>,
    pub(crate) name_override: Option<String>,
    pub(crate) is_deref_return: bool,
    pub(crate) is_async: bool,
}

pub(crate) fn derive_methods(
    args: Vec<NestedMeta>,
    item_impl: ItemImpl,
) -> Result<TokenStream2, syn::Error> {
    let derived = crate::automatically_derived();
    let gdnative_core = crate::crate_gdnative_core();
    let (impl_block, export) = impl_gdnative_expose(item_impl);
    let (impl_generics, _, where_clause) = impl_block.generics.split_for_impl();

    let class_name = export.class_ty;

    let builder = syn::Ident::new("builder", proc_macro2::Span::call_site());

    let args = {
        let mut attr_args_builder = MixinArgsBuilder::new();

        for arg in args {
            if let NestedMeta::Meta(Meta::NameValue(ref pair)) = arg {
                attr_args_builder.add_pair(pair)?;
            } else if let NestedMeta::Meta(Meta::Path(ref path)) = arg {
                attr_args_builder.add_path(path)?;
            } else {
                let msg = format!("Unexpected argument: {arg:?}");
                return Err(syn::Error::new(arg.span(), msg));
            }
        }

        attr_args_builder.done()?
    };

    let non_concrete = find_non_concrete::with_visitor(&impl_block.generics, |v| {
        v.visit_type(&impl_block.self_ty)
    });

    let non_concrete = if non_concrete.is_empty() {
        None
    } else if non_concrete.len() == 1 {
        Some(non_concrete[0])
    } else {
        Some(impl_block.self_ty.span())
    };

    if let Some(span) = non_concrete {
        if matches!(args.mixin, Some(MixinKind::Auto(_))) {
            return Err(syn::Error::new(
                span,
                "non-concrete mixins must be named and manually registered",
            ));
        }
    }

    let methods = export
        .methods
        .into_iter()
        .map(|export_method| {
            let ExportMethod {
                sig,
                export_args,
                ..
            } = &export_method;

            let sig_span = sig.ident.span();

            let name = sig.ident.clone();
            let name_string = export_args
                .name_override
                .clone()
                .unwrap_or_else(|| name.to_string());
            let ret_span = sig.output.span();

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

            let method = wrap_method(&class_name, &impl_block.generics, &export_method)
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

    match args.mixin {
        Some(mixin_kind) => {
            let vis = args.pub_.then(|| quote!(pub));

            let mixin_name = match &mixin_kind {
                MixinKind::Named(ident) => ident.clone(),
                MixinKind::Auto(span) => {
                    return Err(syn::Error::new(
                        *span,
                        "mixins must be named in gdnative v0.11.x",
                    ))
                }
            };

            let body = quote! {
                #derived
                #vis struct #mixin_name {
                    _opaque: #gdnative_core::private::mixin::Opaque,
                }

                #derived
                impl #gdnative_core::private::mixin::Sealed for #mixin_name {}
                #derived
                impl #impl_generics #gdnative_core::export::Mixin<#class_name> for #mixin_name #where_clause {
                    fn register(#builder: &#gdnative_core::export::ClassBuilder<#class_name>) {
                        use #gdnative_core::export::*;

                        #(#methods)*
                    }
                }
            };

            let body = match &mixin_kind {
                MixinKind::Named(_) => body,
                MixinKind::Auto(_) => quote! {
                    const _: () = {
                        #body
                    }
                },
            };

            Ok(quote::quote!(
                #impl_block
                #body
            ))
        }
        None => Ok(quote::quote!(
            #impl_block

            #derived
            impl #impl_generics #gdnative_core::export::NativeClassMethods for #class_name #where_clause {
                fn nativeclass_register(#builder: &#gdnative_core::export::ClassBuilder<Self>) {
                    use #gdnative_core::export::*;

                    #(#methods)*
                }
            }

        )),
    }
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
                            use syn::{punctuated::Punctuated, Lit};
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
                                                    format!("unexpected value for `rpc`: {value}"),
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
                                } else if path.is_ident("async") {
                                    // deref return value
                                    if lit.is_some() {
                                        errors.push(syn::Error::new(
                                            nested_meta.span(),
                                            "`async` does not take any values",
                                        ));
                                    } else if export_args.is_async {
                                        errors.push(syn::Error::new(
                                            nested_meta.span(),
                                            "`async` was set more than once",
                                        ));
                                    } else {
                                        export_args.is_async = true;
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
                    methods_to_export.extend(ExportMethod::strip_parse(
                        &mut method.sig,
                        export_args,
                        &mut errors,
                    ));
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
                            let name = format!("___unused_arg_{i}");

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

            // Ignore the trailing comma
            let _ = input.parse::<Token![,]>();

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

    let export_args = ExportArgs {
        is_old_syntax: false,
        rpc_mode: None,
        name_override: None,
        is_deref_return: is_deref_return.value,
        is_async: false,
    };

    let mut errors = Vec::new();
    let export_method = ExportMethod::strip_parse(&mut signature, export_args, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    wrap_method(
        &class_name,
        &Generics::default(),
        &export_method.expect("ExportMethod is valid"),
    )
    .map_err(|e| vec![e])
}

fn wrap_method(
    class_name: &Type,
    generics: &Generics,
    export_method: &ExportMethod,
) -> Result<TokenStream2, syn::Error> {
    let ExportMethod {
        sig,
        export_args,
        arg_kind,
    } = &export_method;

    let gdnative_core = crate::crate_gdnative_core();
    let automatically_derived = crate::automatically_derived();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let turbofish_ty_generics = ty_generics.as_turbofish();

    let generic_marker_decl = if generics.params.is_empty() {
        quote!(())
    } else {
        quote!(core::marker::PhantomData #ty_generics)
    };

    let generic_marker_ctor = if generics.params.is_empty() {
        quote!(())
    } else {
        quote!(core::marker::PhantomData)
    };

    let sig_span = sig.ident.span();
    let ret_span = sig.output.span();

    let method_name = &sig.ident;

    let declare_arg_list = arg_kind
        .iter()
        .zip(&sig.inputs)
        .filter_map(|(kind, arg)| {
            if let ArgKind::Regular { optional } = kind {
                if let FnArg::Typed(arg) = arg {
                    let span = arg.span();
                    let maybe_opt = if *optional {
                        Some(quote_spanned!(span => #[opt]))
                    } else {
                        None
                    };
                    Some(quote_spanned!(span => #maybe_opt #arg))
                } else {
                    unreachable!("regular arguments should always be FnArg::Typed")
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let destructure_arg_list = arg_kind
        .iter()
        .zip(&sig.inputs)
        .filter_map(|(kind, arg)| {
            if matches!(kind, ArgKind::Regular { .. }) {
                if let FnArg::Typed(arg) = arg {
                    Some(&arg.pat)
                } else {
                    unreachable!("regular arguments should always be FnArg::Typed")
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let is_async = export_args.is_async || sig.asyncness.is_some();
    let mut map_method = None;

    let invoke_arg_list = arg_kind
        .iter()
        .zip(&sig.inputs)
        .map(|(kind, arg)| match kind {
            ArgKind::Receiver => {
                if let FnArg::Receiver(receiver) = arg {
                    map_method = Some(if receiver.reference.is_some() {
                        if receiver.mutability.is_some() {
                            syn::Ident::new("map_mut", sig_span)
                        } else {
                            syn::Ident::new("map", sig_span)
                        }
                    } else {
                        syn::Ident::new("map_owned", sig_span)
                    });
                }

                Ok(quote_spanned! { sig_span => __rust_val })
            }
            ArgKind::Base => Ok(quote_spanned! { sig_span => OwnerArg::from_safe_ref(__base) }),
            ArgKind::AsyncCtx => Ok(quote_spanned! { sig_span => __ctx }),
            ArgKind::Regular { .. } => match arg {
                FnArg::Receiver(_) => {
                    unreachable!("receivers cannot be regular arguments")
                }
                FnArg::Typed(arg) => {
                    let pat = &arg.pat;
                    Ok(quote_spanned! { sig_span => #pat })
                }
            },
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let map_method = map_method.unwrap_or_else(|| syn::Ident::new("map", sig_span));

    let recover = if export_args.is_deref_return {
        quote_spanned! { ret_span => std::ops::Deref::deref(&ret) }
    } else {
        quote_spanned! { ret_span => ret }
    };

    let impl_body = if is_async {
        let gdnative_async = crate::crate_gdnative_async();

        quote_spanned! { sig_span =>
            #automatically_derived
            impl #impl_generics #gdnative_async::StaticArgsAsyncMethod<#class_name> for ThisMethod #ty_generics #where_clause {
                type Args = Args #ty_generics;

                fn spawn_with(
                    &self,
                    __spawner: #gdnative_async::Spawner::<'_, #class_name, Self::Args>,
                ) {
                    __spawner.spawn(move |__ctx, __this, __args| {
                        let __future = __this
                            .#map_method(move |__rust_val, __base| {
                                let Args { #(#destructure_arg_list,)* __generic_marker } = __args;

                                #[allow(unused_unsafe)]
                                unsafe {
                                    Some(<#class_name>::#method_name(
                                        #(#invoke_arg_list,)*
                                    ))
                                }
                            })
                            .unwrap_or_else(|err| {
                                #gdnative_core::godot_error!("gdnative-core: method call failed with error: {}", err);
                                #gdnative_core::godot_error!("gdnative-core: check module level documentation on gdnative::user_data for more information");
                                None
                            });

                        async move {
                            if let Some(__future) = __future {
                                let ret = __future.await;
                                #gdnative_core::core_types::OwnedToVariant::owned_to_variant(#recover)
                            } else {
                                #gdnative_core::core_types::Variant::nil()
                            }
                        }
                    });
                }

                fn site() -> Option<#gdnative_core::log::Site<'static>> {
                    Some(#gdnative_core::godot_site!(#class_name::#method_name))
                }
            }

            #gdnative_async::Async::new(#gdnative_async::StaticArgs::new(ThisMethod #turbofish_ty_generics {
                _marker: #generic_marker_ctor,
            }))
        }
    } else {
        quote_spanned! { sig_span =>
            #automatically_derived
            impl #impl_generics #gdnative_core::export::StaticArgsMethod<#class_name> for ThisMethod #ty_generics #where_clause {
                type Args = Args #ty_generics;
                fn call(
                    &self,
                    __this: TInstance<'_, #class_name, #gdnative_core::object::ownership::Shared>,
                    Args { #(#destructure_arg_list,)* __generic_marker }: Self::Args,
                ) -> #gdnative_core::core_types::Variant {
                    __this
                        .#map_method(|__rust_val, __base| {
                            #[allow(unused_unsafe)]
                            unsafe {
                                let ret = <#class_name>::#method_name(
                                    #(#invoke_arg_list,)*
                                );
                                #gdnative_core::core_types::OwnedToVariant::owned_to_variant(#recover)
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

            #gdnative_core::export::StaticArgs::new(ThisMethod #turbofish_ty_generics {
                _marker: #generic_marker_ctor,
            })
        }
    };

    // Necessary standard traits have to be implemented manually because the default derive isn't smart enough.
    let output = quote_spanned! { sig_span =>
        {
            struct ThisMethod #ty_generics #where_clause {
                _marker: #generic_marker_decl,
            }

            impl #impl_generics Copy for ThisMethod #ty_generics #where_clause {}
            impl #impl_generics Clone for ThisMethod #ty_generics #where_clause {
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl #impl_generics Default for ThisMethod #ty_generics #where_clause {
                fn default() -> Self {
                    Self {
                        _marker: #generic_marker_ctor,
                    }
                }
            }

            unsafe impl #impl_generics Send for ThisMethod #ty_generics #where_clause {}
            unsafe impl #impl_generics Sync for ThisMethod #ty_generics #where_clause {}

            use #gdnative_core::export::{NativeClass, OwnerArg};
            use #gdnative_core::object::{Instance, TInstance};
            use #gdnative_core::derive::FromVarargs;

            #[derive(FromVarargs)]
            #automatically_derived
            struct Args #ty_generics #where_clause {
                #(#declare_arg_list,)*

                #[skip]
                __generic_marker: #generic_marker_decl,
            }

            #impl_body
        }
    };

    Ok(output)
}

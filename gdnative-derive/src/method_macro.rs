use syn::{FnArg, ImplItem, ItemImpl, Signature, Pat, PatIdent, Type};

use proc_macro::TokenStream;
use syn::export::Span;
use std::boxed::Box;

pub(crate) struct ClassMethodExport {
    pub(crate) class_ty: Box<Type>,
    pub(crate) methods: Vec<Signature>,
}

/// Parse the input.
///
/// Returns the TokenStream of the impl block together with a description of methods to export.
pub(crate) fn parse_method_export(
    _meta: TokenStream,
    input: TokenStream,
) -> (ItemImpl, ClassMethodExport) {
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

    let mut methods_to_export = Vec::<Signature>::new();

    // extract all methods that have the #[export] attribute.
    // add all items back to the impl block again.
    for func in ast.items {
        let item = match func {
            ImplItem::Method(mut method) => {
                // only allow the "outer" style, aka #[thing] item.
                let attribute_pos = method.attrs.iter().position(|attr| {
                    let correct_style = match attr.style {
                        syn::AttrStyle::Outer => true,
                        _ => false,
                    };

                    for path in attr.path.segments.iter() {
                        if path.ident.to_string() == "export" {
                            return correct_style;
                        }
                    }

                    false
                });

                if let Some(idx) = attribute_pos {
                    // TODO renaming? rpc modes?
                    let _attr = method.attrs.remove(idx);

                    methods_to_export.push(method.sig.clone());
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
            let generics = &method.generics;

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
            method.unsafety = None;

            export.methods.push(method);
        }
    }

    (result, export)
}

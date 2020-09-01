use syn::{spanned::Spanned, AttributeArgs, ItemFn, Meta, NestedMeta};

use proc_macro::TokenStream;

pub struct ProfiledAttrArgs {
    pub tag: Option<String>,
}

#[derive(Default)]
pub struct ProfiledAttrArgsBuilder {
    tag: Option<String>,
    errors: Vec<syn::Error>,
}

impl<'a> Extend<&'a syn::NestedMeta> for ProfiledAttrArgsBuilder {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a syn::NestedMeta>,
    {
        for meta in iter.into_iter() {
            let pair = match meta {
                NestedMeta::Meta(Meta::NameValue(name_value)) => name_value,
                _ => {
                    self.errors
                        .push(syn::Error::new(meta.span(), "expecting name-value pair"));
                    continue;
                }
            };

            let name = pair
                .path
                .get_ident()
                .expect("should be single identifier")
                .to_string();
            match name.as_str() {
                "tag" => {
                    let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                        lit_str.value()
                    } else {
                        self.errors.push(syn::Error::new(
                            pair.lit.span(),
                            "tag value is not a string literal",
                        ));
                        continue;
                    };

                    if let Some(old) = self.tag.replace(string) {
                        self.errors.push(syn::Error::new(
                            pair.lit.span(),
                            format!("there is already a tag set: {:?}", old),
                        ));
                    }
                }
                _ => {
                    self.errors
                        .push(syn::Error::new(pair.span(), "unexpected argument"));
                }
            }
        }
    }
}

impl ProfiledAttrArgsBuilder {
    pub fn done(self) -> Result<ProfiledAttrArgs, Vec<syn::Error>> {
        if self.errors.is_empty() {
            Ok(ProfiledAttrArgs { tag: self.tag })
        } else {
            Err(self.errors)
        }
    }
}

pub(crate) fn derive_profiled(meta: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(meta as AttributeArgs);
    let mut input = parse_macro_input!(input as ItemFn);

    let args = {
        let mut args_builder = ProfiledAttrArgsBuilder::default();
        args_builder.extend(args.iter());
        match args_builder.done() {
            Ok(args) => args,
            Err(errors) => {
                return errors
                    .into_iter()
                    .map(|e| TokenStream::from(e.to_compile_error()))
                    .collect()
            }
        }
    };

    let tag = match args.tag {
        Some(tag) => quote!(#tag),
        None => {
            let ident = input.sig.ident.to_string();

            quote! {
                &format!(
                    "{}/{}",
                    module_path!()
                        .split("::")
                        .last()
                        .expect("module path should be non-empty"),
                    #ident,
                )
            }
        }
    };

    let stmts = std::mem::take(&mut input.block.stmts);
    input.block = Box::new(parse_quote!({
        ::gdnative::nativescript::profiling::profile(::gdnative::profile_sig!(#tag), move || {
            #(#stmts)*
        })
    }));

    quote!(#input).into()
}

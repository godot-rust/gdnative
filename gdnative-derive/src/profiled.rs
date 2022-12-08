use syn::{spanned::Spanned, AttributeArgs, ItemFn, Meta, NestedMeta};

use proc_macro2::TokenStream as TokenStream2;

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
                            format!("there is already a tag set: {old:?}"),
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

pub(crate) fn derive_profiled(
    args: AttributeArgs,
    mut item_fn: ItemFn,
) -> Result<TokenStream2, syn::Error> {
    let args = {
        let mut args_builder = ProfiledAttrArgsBuilder::default();
        args_builder.extend(args.iter());
        match args_builder.done() {
            Ok(args) => args,
            Err(mut errors) => {
                // Combine the errors into one erorr
                let first_error = errors.remove(0);
                let combined_errors = errors.into_iter().fold(first_error, |mut errors, error| {
                    errors.combine(error);
                    errors
                });
                return Err(combined_errors);
            }
        }
    };

    let tag = match args.tag {
        Some(tag) => quote!(#tag),
        None => {
            let ident = item_fn.sig.ident.to_string();

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

    let stmts = std::mem::take(&mut item_fn.block.stmts);
    item_fn.block = Box::new(parse_quote!({
        ::gdnative::profiler::profile(
            ::gdnative::profiler::profile_sig!(#tag), move || {
            #(#stmts)*
        })
    }));

    Ok(quote!(#item_fn))
}

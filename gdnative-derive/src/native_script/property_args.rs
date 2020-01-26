pub struct PropertyAttrArgs {
    pub path: Option<String>,
    pub default: Option<syn::Lit>,
}

#[derive(Default)]
pub struct PropertyAttrArgsBuilder {
    path: Option<String>,
    default: Option<syn::Lit>,
}

impl<'a> Extend<&'a syn::MetaNameValue> for PropertyAttrArgsBuilder {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a syn::MetaNameValue>,
    {
        for pair in iter.into_iter() {
            let name = pair
                .path
                .get_ident()
                .expect("should be single identifier")
                .to_string();
            match name.as_str() {
                "default" => {
                    if let Some(old) = self.default.replace(pair.lit.clone()) {
                        panic!("there is already a default value set: {:?}", old);
                    }
                }
                "path" => {
                    let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                        lit_str.value()
                    } else {
                        panic!("path value is not a string literal");
                    };

                    if let Some(old) = self.path.replace(string) {
                        panic!("there is already a path set: {:?}", old);
                    }
                }
                _ => panic!("unexpected argument: {}", &name),
            }
        }
    }
}

impl PropertyAttrArgsBuilder {
    pub fn done(self) -> PropertyAttrArgs {
        PropertyAttrArgs {
            path: self.path,
            default: self.default,
        }
    }
}

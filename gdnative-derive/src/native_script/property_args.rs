pub struct PropertyAttrArgs {
    pub path: Option<String>,
    pub default: Option<syn::Lit>,
    pub before_get: Option<syn::Path>,
    pub after_get: Option<syn::Path>,
    pub before_set: Option<syn::Path>,
    pub after_set: Option<syn::Path>,
}

#[derive(Default)]
pub struct PropertyAttrArgsBuilder {
    path: Option<String>,
    default: Option<syn::Lit>,
    before_get: Option<syn::Path>,
    after_get: Option<syn::Path>,
    before_set: Option<syn::Path>,
    after_set: Option<syn::Path>,
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
                "before_get" => {
                    let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                        lit_str.value()
                    } else {
                        panic!("before_get value is not a string literal");
                    };

                    let path = syn::parse_str::<syn::Path>(string.as_str())
                        .expect("Invalid path expression.");
                    if let Some(old) = self.before_get.replace(path) {
                        panic!("there is already a before_get value set: {:?}", old);
                    }
                }
                "after_get" => {
                    let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                        lit_str.value()
                    } else {
                        panic!("after_get value is not a string literal");
                    };

                    let path = syn::parse_str::<syn::Path>(string.as_str())
                        .expect("Invalid path expression.");
                    if let Some(old) = self.after_get.replace(path) {
                        panic!("there is already a after_get value set: {:?}", old);
                    }
                }
                "before_set" => {
                    let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                        lit_str.value()
                    } else {
                        panic!("before_set value is not a string literal");
                    };

                    let path = syn::parse_str::<syn::Path>(string.as_str())
                        .expect("Invalid path expression.");
                    if let Some(old) = self.before_set.replace(path) {
                        panic!("there is already a before_set value set: {:?}", old);
                    }
                }
                "after_set" => {
                    let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                        lit_str.value()
                    } else {
                        panic!("after_set value is not a string literal");
                    };

                    let path = syn::parse_str::<syn::Path>(string.as_str())
                        .expect("Invalid path expression.");
                    if let Some(old) = self.after_set.replace(path) {
                        panic!("there is already a after_set value set: {:?}", old);
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
            before_get: self.before_get,
            after_get: self.after_get,
            before_set: self.before_set,
            after_set: self.after_set,
        }
    }
}

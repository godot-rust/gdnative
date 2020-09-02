#[derive(Debug)]
pub struct PropertyAttrArgs {
    pub path: Option<String>,
    pub default: Option<syn::Lit>,
    pub update_config_warning_on_set: bool,
}

#[derive(Default)]
pub struct PropertyAttrArgsBuilder {
    path: Option<String>,
    default: Option<syn::Lit>,
    update_config_warning_on_set: bool,
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
                "update_config_warning_on_set" => {
                    panic!("Found it.");
                }
                _ => panic!("unexpected argument: {}", &name),
            }
        }
    }
}

impl<'a> Extend<&'a syn::Path> for PropertyAttrArgsBuilder {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a syn::Path>,
    {
        for path in iter.into_iter() {
            let name = path
                .get_ident()
                .expect("should be single identifier")
                .to_string();
            match name.as_str() {
                "update_config_warning_on_set" => {
                    if self.update_config_warning_on_set {
                        panic!("The \"update_config_warning_on_set\" property modifier was used more than once.");
                    }
                    self.update_config_warning_on_set = true;
                }
                _ => panic!("Unexpected argument: {}", &name),
            }
        }
    }
}

impl PropertyAttrArgsBuilder {
    pub fn done(self) -> PropertyAttrArgs {
        PropertyAttrArgs {
            path: self.path,
            default: self.default,
            update_config_warning_on_set: self.update_config_warning_on_set,
        }
    }
}

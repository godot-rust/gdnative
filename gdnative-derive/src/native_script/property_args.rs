use syn::spanned::Spanned;

pub struct PropertyAttrArgs {
    pub path: Option<String>,
    pub default: Option<syn::Lit>,
    pub hint: Option<syn::Path>,
    pub before_get: Option<syn::Path>,
    pub after_get: Option<syn::Path>,
    pub before_set: Option<syn::Path>,
    pub after_set: Option<syn::Path>,
}

#[derive(Default)]
pub struct PropertyAttrArgsBuilder {
    path: Option<String>,
    default: Option<syn::Lit>,
    hint: Option<syn::Path>,
    before_get: Option<syn::Path>,
    after_get: Option<syn::Path>,
    before_set: Option<syn::Path>,
    after_set: Option<syn::Path>,
}

impl PropertyAttrArgsBuilder {
    pub fn add_pair(&mut self, pair: &syn::MetaNameValue) -> Result<(), syn::Error> {
        let path_span = pair.lit.span();
        let invalid_value_path = |_| {
            syn::Error::new(
                path_span,
                "Unexpected input, expected a double quoted string: \"path::to::something\"",
            )
        };

        let name = pair
            .path
            .get_ident()
            .expect("should be single identifier")
            .to_string();
        match name.as_str() {
            "default" => {
                if let Some(old) = self.default.replace(pair.lit.clone()) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a default value set: {:?}", old),
                    ));
                }
            }
            "path" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "path value is not a string literal".to_string(),
                    ));
                };

                if let Some(old) = self.path.replace(string) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a path set: {:?}", old),
                    ));
                }
            }
            "hint" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "hint value is not a string literal".to_string(),
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.hint.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a hint value set: {:?}", old),
                    ));
                }
            }
            "before_get" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "before_get value is not a string literal".to_string(),
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.before_get.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a before_get value set: {:?}", old),
                    ));
                }
            }
            "after_get" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "after_get value is not a string literal".to_string(),
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.after_get.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a after_get value set: {:?}", old),
                    ));
                }
            }
            "before_set" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "before_set value is not a string literal".to_string(),
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.before_set.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a before_set value set: {:?}", old),
                    ));
                }
            }
            "after_set" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "after_set value is not a string literal".to_string(),
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.after_set.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a after_set value set: {:?}", old),
                    ));
                }
            }
            _ => {
                return Err(syn::Error::new(
                    pair.span(),
                    format!("unexpected argument: {}", &name),
                ))
            }
        }

        Ok(())
    }
}

impl PropertyAttrArgsBuilder {
    pub fn done(self) -> PropertyAttrArgs {
        PropertyAttrArgs {
            path: self.path,
            default: self.default,
            hint: self.hint,
            before_get: self.before_get,
            after_get: self.after_get,
            before_set: self.before_set,
            after_set: self.after_set,
        }
    }
}

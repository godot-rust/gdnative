use syn::spanned::Spanned;

#[derive(Debug)]
pub enum PropertyGet {
    Default,
    Owned(syn::Path),
    Ref(syn::Path),
}

#[derive(Debug)]
pub enum PropertySet {
    Default,
    WithPath(syn::Path),
}

pub struct PropertyAttrArgs {
    pub ty: syn::Type,
    pub path: Option<String>,
    pub default: Option<syn::Lit>,
    pub hint: Option<syn::Path>,
    pub before_get: Option<syn::Path>,
    pub get: Option<PropertyGet>,
    pub after_get: Option<syn::Path>,
    pub before_set: Option<syn::Path>,
    pub set: Option<PropertySet>,
    pub after_set: Option<syn::Path>,
    pub no_editor: bool,
}

pub struct PropertyAttrArgsBuilder {
    ty: syn::Type,
    path: Option<String>,
    default: Option<syn::Lit>,
    hint: Option<syn::Path>,
    before_get: Option<syn::Path>,
    get: Option<PropertyGet>,
    after_get: Option<syn::Path>,
    before_set: Option<syn::Path>,
    set: Option<PropertySet>,
    after_set: Option<syn::Path>,
    no_editor: bool,
}

impl PropertyAttrArgsBuilder {
    pub fn new(ty: &syn::Type) -> Self {
        Self {
            ty: ty.clone(),
            path: None,
            default: None,
            hint: None,
            before_get: None,
            get: None,
            after_get: None,
            before_set: None,
            set: None,
            after_set: None,
            no_editor: false,
        }
    }

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
                        format!(
                            "there is already a 'default' attribute with value: {:?}",
                            old
                        ),
                    ));
                }
            }
            "path" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'path' value is not a string literal",
                    ));
                };

                if let Some(old) = self.path.replace(string) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a 'path' attribute with value: {:?}", old),
                    ));
                }
            }
            "hint" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'hint' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.hint.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a 'hint' attribute with value: {:?}", old),
                    ));
                }
            }
            "before_get" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'before_get' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.before_get.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!(
                            "there is already a 'before_get' attribute with value: {:?}",
                            old
                        ),
                    ));
                }
            }
            "get" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'get' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                let get = PropertyGet::Owned(path);
                if let Some(old) = self.get.replace(get) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a 'get' attribute with value: {:?}", old),
                    ));
                }
            }
            "get_ref" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'get_ref' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                let get_ref = PropertyGet::Ref(path);
                if let Some(old) = self.get.replace(get_ref) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!(
                            "there is already a 'get_ref' attribute with value: {:?}",
                            old
                        ),
                    ));
                }
            }
            "after_get" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'after_get' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.after_get.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!(
                            "there is already a 'after_get' attribute with value: {:?}",
                            old
                        ),
                    ));
                }
            }
            "before_set" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'before_set' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.before_set.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!(
                            "there is already a 'before_set' attribute with value: {:?}",
                            old
                        ),
                    ));
                }
            }
            "set" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'set' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                let set = PropertySet::WithPath(path);
                if let Some(old) = self.set.replace(set) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!("there is already a 'set' attribute with value: {:?}", old),
                    ));
                }
            }
            "after_set" => {
                let string = if let syn::Lit::Str(lit_str) = &pair.lit {
                    lit_str.value()
                } else {
                    return Err(syn::Error::new(
                        pair.span(),
                        "'after_set' value is not a string literal",
                    ));
                };

                let path =
                    syn::parse_str::<syn::Path>(string.as_str()).map_err(invalid_value_path)?;
                if let Some(old) = self.after_set.replace(path) {
                    return Err(syn::Error::new(
                        pair.span(),
                        format!(
                            "there is already a 'after_set' attribute with value: {:?}",
                            old
                        ),
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

    pub fn add_path(&mut self, path: &syn::Path) -> Result<(), syn::Error> {
        if path.is_ident("no_editor") {
            self.no_editor = true;
        } else if path.is_ident("get") {
            if let Some(get) = self.get.replace(PropertyGet::Default) {
                return Err(syn::Error::new(
                    path.span(),
                    format!("there is already a 'get' attribute with value: {:?}", get),
                ));
            }
        } else if path.is_ident("set") {
            if let Some(set) = self.set.replace(PropertySet::Default) {
                return Err(syn::Error::new(
                    path.span(),
                    format!("there is already a 'set' attribute with value: {:?}", set),
                ));
            }
        } else {
            return Err(syn::Error::new(
                path.span(),
                format!("unexpected argument: {:?}", path.get_ident()),
            ));
        }

        Ok(())
    }
}

impl PropertyAttrArgsBuilder {
    pub fn done(self) -> PropertyAttrArgs {
        PropertyAttrArgs {
            ty: self.ty,
            path: self.path,
            default: self.default,
            hint: self.hint,
            before_get: self.before_get,
            get: self.get,
            after_get: self.after_get,
            before_set: self.before_set,
            set: self.set,
            after_set: self.after_set,
            no_editor: self.no_editor,
        }
    }
}

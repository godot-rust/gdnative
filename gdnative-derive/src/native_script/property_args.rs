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
    pub get: Option<PropertyGet>,
    pub set: Option<PropertySet>,
    pub no_editor: bool,
}

pub struct PropertyAttrArgsBuilder {
    ty: syn::Type,
    path: Option<String>,
    default: Option<syn::Lit>,
    hint: Option<syn::Path>,
    get: Option<PropertyGet>,
    set: Option<PropertySet>,
    no_editor: bool,
}

impl PropertyAttrArgsBuilder {
    pub fn new(ty: &syn::Type) -> Self {
        Self {
            ty: ty.clone(),
            path: None,
            default: None,
            hint: None,
            get: None,
            set: None,
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
            get: self.get,
            set: self.set,
            no_editor: self.no_editor,
        }
    }
}

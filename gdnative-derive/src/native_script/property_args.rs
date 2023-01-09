use proc_macro2::Span;
use std::fmt::Debug;
use syn::spanned::Spanned;

use crate::syntax::rpc_mode::RpcMode;

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
    pub rpc_mode: Option<RpcMode>,
    pub no_editor: bool,
}

pub struct PropertyAttrArgsBuilder {
    ty: syn::Type,
    path: Option<String>,
    default: Option<syn::Lit>,
    hint: Option<syn::Path>,
    get: Option<PropertyGet>,
    set: Option<PropertySet>,
    rpc_mode: Option<RpcMode>,
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
            rpc_mode: None,
            no_editor: false,
        }
    }

    /// Error returned when a value are set twice
    /// e.g. #[property(set = "Self::set_foo", set = "Self::set_foo_again")]
    fn err_prop_already_set<T: Debug>(span: Span, prop: &str, old: &T) -> syn::Error {
        syn::Error::new(
            span,
            format!("there is already a '{prop}' attribute with value: {old:?}",),
        )
    }

    // Error returned when the attr value is not a string literal (i.e. not `LitStr`)
    fn err_attr_not_a_string_literal(span: Span, attr: &str) -> syn::Error {
        syn::Error::new(span, format!("'{attr}' value is not a string literal"))
    }

    /// Convert `Lit` to `LitStr`
    fn extract_lit_str(lit: &syn::Lit) -> Option<&syn::LitStr> {
        if let syn::Lit::Str(lit_str) = lit {
            Some(lit_str)
        } else {
            None
        }
    }

    pub fn add_pair(&mut self, pair: &syn::MetaNameValue) -> Result<(), syn::Error> {
        // Update property with input value.
        // Return error when there is already a value set
        macro_rules! update_prop {
            ($prop:ident, $val:expr) => {
                if let Some(old) = self.$prop.replace($val) {
                    return Err(Self::err_prop_already_set(
                        pair.span(),
                        stringify!($prop),
                        &old,
                    ));
                }
            };
        }

        // Convert input literal to a `syn::Path`
        let parse_path = |prop_name| {
            Self::extract_lit_str(&pair.lit)
                .ok_or_else(|| Self::err_attr_not_a_string_literal(pair.span(), prop_name))?
                .parse::<syn::Path>()
                .map_err(|_| {
                    syn::Error::new(
                        pair.lit.span(),
                        "Unexpected input, expected a double quoted string: \"path::to::something\"",
                    )
                })
        };
        // Convert input to `syn::Path`, and then update property
        macro_rules! process_path_input {
            ($prop:ident$(, $($mapping:path),*)?) => {{
                let path = parse_path(stringify!($prop))?;
                $(
                    $(let path = $mapping(path);)*
                )?
                update_prop!($prop, path);
            }};
        }

        let name = pair
            .path
            .get_ident()
            .expect("should be single identifier")
            .to_string();
        match name.as_str() {
            "default" => update_prop!(default, pair.lit.clone()),
            "path" => {
                let path = Self::extract_lit_str(&pair.lit)
                    .ok_or_else(|| Self::err_attr_not_a_string_literal(pair.span(), "path"))?;
                update_prop!(path, path.value());
            }
            "hint" => process_path_input!(hint),
            "get" => process_path_input!(get, PropertyGet::Owned),
            "get_ref" => process_path_input!(get, PropertyGet::Ref),
            "set" => process_path_input!(set, PropertySet::WithPath),
            "rpc" => {
                let rpc = Self::extract_lit_str(&pair.lit)
                    .ok_or_else(|| Self::err_attr_not_a_string_literal(pair.span(), "rpc"))?;
                let rpc = rpc.value();
                let rpc = RpcMode::parse(&rpc).ok_or_else(|| {
                    syn::Error::new(
                        pair.lit.span(),
                        format!("unexpected value for `rpc`: {rpc}"),
                    )
                })?;
                update_prop!(rpc_mode, rpc)
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
                return Err(Self::err_prop_already_set(path.span(), "get", &get));
            }
        } else if path.is_ident("set") {
            if let Some(set) = self.set.replace(PropertySet::Default) {
                return Err(Self::err_prop_already_set(path.span(), "set", &set));
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
            rpc_mode: self.rpc_mode,
            no_editor: self.no_editor,
        }
    }
}

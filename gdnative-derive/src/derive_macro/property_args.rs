pub struct PropertyAttrArgs {
    pub default: syn::Lit,
}

#[derive(Default)]
pub struct PropertyAttrArgsBuilder {
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
                _ => panic!("unexpected argument: {}", &name),
            }
        }
    }
}

impl PropertyAttrArgsBuilder {
    pub fn done(self) -> PropertyAttrArgs {
        PropertyAttrArgs {
            default: self.default.expect("`default` value is required"),
        }
    }
}

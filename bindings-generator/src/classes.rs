use crate::api::*;
use crate::class_docs::GodotXmlDocs;
use crate::methods;
use crate::special_methods;

use heck::ToPascalCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::collections::HashMap;

pub(crate) fn generate_class_struct(class: &GodotClass, class_doc: TokenStream) -> TokenStream {
    let class_name = format_ident!("{}", &class.name);

    // dead_code: 'this' might not be read
    // mod private: hide the type in the #module_name module, export it only in gdnative::api
    quote! {
        pub(crate) mod private {
            #class_doc
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            pub struct #class_name {
                #[allow(dead_code)]
                pub(crate) this: super::RawObject<Self>,
            }
        }
        use private::#class_name;
    }
}

pub(crate) fn generate_class_impl(
    class: &GodotClass,
    icalls: &mut HashMap<String, methods::MethodSig>,
    docs: Option<&GodotXmlDocs>,
) -> TokenStream {
    let class_singleton = if class.singleton {
        special_methods::generate_singleton_getter(class)
    } else {
        Default::default()
    };

    let class_singleton_getter = if class.name == "GDNativeLibrary" {
        special_methods::generate_gdnative_library_singleton_getter(class)
    } else {
        Default::default()
    };

    let class_instanciable = if class.instantiable {
        special_methods::generate_ctor(class)
    } else {
        Default::default()
    };

    let class_methods = methods::generate_methods(class, icalls, docs);

    let class_name = format_ident!("{}", class.name);
    quote! {
        impl #class_name {
            #class_singleton
            #class_singleton_getter
            #class_instanciable
            #class_methods
        }
    }
}

pub(crate) fn generate_class_constants(class: &GodotClass) -> TokenStream {
    assert!(
        !class.constants.is_empty(),
        "Only call on class with constants."
    );

    let mut constants = TokenStream::new();

    let mut class_constants: Vec<(&ConstantName, &ConstantValue)> =
        class.constants.iter().collect();
    class_constants.sort_by(constant_sorter);

    for (name, value) in class_constants {
        let name = format_ident!("{}", name);
        let constant = quote! {
            pub const #name: i64 = #value;
        };
        constants.extend(constant);
    }

    let class_name = format_ident!("{}", &class.name);

    quote! {
        #[doc="Constants"]
        #[allow(non_upper_case_globals)]
        impl #class_name {
            #constants
        }
    }
}

pub(crate) fn generate_enums(class: &GodotClass) -> TokenStream {
    let mut enums: Vec<&Enum> = class.enums.iter().collect();
    enums.sort();
    let enums = enums.iter().map(|e| {
        let enum_name = generate_enum_name(&class.name, &e.name);
        let typ_name = format_ident!("{}", enum_name);

        let mut values: Vec<_> = e.values.iter().collect();
        values.sort_by(constant_sorter);

        let consts = values.iter().map(|(key, val)| {
            let key = key.to_uppercase();
            let variant = format_ident!("{}", key);
            quote! {
                pub const #variant: #typ_name = #typ_name(#val);
            }
        });

        quote! {
            #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct #typ_name(pub i64);

            impl #typ_name {
                #(#consts)*
            }
            impl From<i64> for #typ_name {
                #[inline]
                fn from(v: i64) -> Self {
                    Self(v)
                }
            }
            impl From<#typ_name> for i64 {
                #[inline]
                fn from(v: #typ_name) -> Self {
                    v.0
                }
            }

            impl FromVariant for #typ_name {
                #[inline]
                fn from_variant(v: &Variant) -> Result<Self, FromVariantError> {
                    i64::from_variant(v).map(Self::from)
                }
            }
        }
    });

    quote! {
        #(#enums)*
    }
}

pub(crate) fn generate_enum_name(class_name: &str, enum_name: &str) -> String {
    // In order to not pollute the API with more Result types,
    // rename the Result enum used by Search to SearchResult.
    // to_pascal_case() is used to make the enums more rust-like.
    // DOFBlurQuality => DofBlurQuality
    match enum_name {
        "Result" => {
            let mut res = String::from(class_name);
            res.push_str(enum_name);
            res.to_pascal_case()
        }
        _ => enum_name.to_pascal_case(),
    }
}

// Ensures deterministic order of constants, not dependent on inner hash-map or sort workings
fn constant_sorter(a: &(&String, &i64), b: &(&String, &i64)) -> std::cmp::Ordering {
    Ord::cmp(a.1, b.1) // first, sort by integer value
        .then(Ord::cmp(a.0, b.0)) // and if equal, by name
}

use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{GenericParam, Generics};

use crate::utils::extend_bounds::{with_visitor, BoundsVisitor};
use crate::variant::repr::StructRepr;

use super::repr::{EnumRepr, Field, Repr, VariantRepr};
use super::Direction;

pub(crate) fn extend_bounds(
    generics: Generics,
    repr: &Repr,
    bound: &syn::Path,
    dir: Direction,
) -> Generics {
    with_visitor(generics, Some(bound), None, |visitor| {
        // iterate through parsed variant representations and visit the types of each field
        fn visit_var_repr<'ast>(
            visitor: &mut BoundsVisitor<'ast>,
            repr: &'ast VariantRepr,
            dir: Direction,
        ) {
            match repr {
                VariantRepr::Unit(_) => {}
                VariantRepr::Tuple(tys) => {
                    for Field { ty, attr, .. } in tys.iter() {
                        if !attr.skip_bounds(dir) {
                            visitor.visit_type(ty);
                        }
                    }
                }
                VariantRepr::Struct(fields) => {
                    for Field { ty, attr, .. } in fields.iter() {
                        if !attr.skip_bounds(dir) {
                            visitor.visit_type(ty);
                        }
                    }
                }
            }
        }

        match repr {
            Repr::Enum(EnumRepr { ref variants, .. }) => {
                for (_, var_repr) in variants.iter() {
                    visit_var_repr(visitor, var_repr, dir);
                }
            }
            Repr::Struct(StructRepr(var_repr)) => {
                visit_var_repr(visitor, var_repr, dir);
            }
        }
    })
}

pub(crate) fn remove_bounds(mut generics: Generics) -> Generics {
    for param in generics.params.iter_mut() {
        match param {
            GenericParam::Type(ty) => {
                ty.colon_token = None;
                ty.bounds = Punctuated::new();
            }
            GenericParam::Lifetime(lt) => {
                lt.colon_token = None;
                lt.bounds = Punctuated::new();
            }
            GenericParam::Const(_) => {}
        }
    }
    generics
}

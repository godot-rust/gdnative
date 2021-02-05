use syn::visit::Visit;
use syn::Generics;

use crate::extend_bounds::{with_visitor, BoundsVisitor};

use super::repr::{Field, Repr, VariantRepr};
use super::Direction;

pub(crate) fn extend_bounds(
    generics: Generics,
    repr: &Repr,
    bound: &syn::Path,
    dir: Direction,
) -> Generics {
    with_visitor(generics, bound, |visitor| {
        // iterate through parsed variant representations and visit the types of each field
        fn visit_var_repr<'ast>(
            visitor: &mut BoundsVisitor<'ast>,
            repr: &'ast VariantRepr,
            dir: Direction,
        ) {
            match repr {
                VariantRepr::Unit => {}
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
            Repr::Enum(ref variants) => {
                for (_, var_repr) in variants.iter() {
                    visit_var_repr(visitor, var_repr, dir);
                }
            }
            Repr::Struct(var_repr) => {
                visit_var_repr(visitor, var_repr, dir);
            }
        }
    })
}

use std::collections::HashSet;

use syn::visit::{self, Visit};
use syn::{Generics, Ident, Type, TypePath};

use super::repr::{Field, Repr, VariantRepr};
use super::Direction;

pub(crate) fn extend_bounds(
    generics: Generics,
    repr: &Repr,
    bound: &syn::Path,
    dir: Direction,
) -> Generics {
    // recursively visit all the field types to find what types should be bounded
    struct Visitor<'ast> {
        all_type_params: HashSet<Ident>,
        used: HashSet<&'ast TypePath>,
    }

    impl<'ast> Visit<'ast> for Visitor<'ast> {
        fn visit_type_path(&mut self, type_path: &'ast TypePath) {
            let path = &type_path.path;
            if let Some(seg) = path.segments.last() {
                if seg.ident == "PhantomData" {
                    // things inside PhantomDatas doesn't need to be bounded, so stopping
                    // recursive visit here
                    return;
                }
            }
            if let Some(seg) = path.segments.first() {
                if self.all_type_params.contains(&seg.ident) {
                    // if the first segment of the type path is a known type variable, then this
                    // is likely an associated type
                    // TODO: what about cases like <Foo<T> as Trait>::A? Maybe too fringe to be
                    // useful? serde_derive can't seem to parse these either. Probably good enough.
                    self.used.insert(type_path);
                }
            }
            visit::visit_path(self, &type_path.path);
        }
    }

    let all_type_params = generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect();

    let mut visitor = Visitor {
        all_type_params: all_type_params,
        used: HashSet::new(),
    };

    // iterate through parsed variant representations and visit the types of each field
    fn visit_var_repr<'ast>(visitor: &mut Visitor<'ast>, repr: &'ast VariantRepr, dir: Direction) {
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
                visit_var_repr(&mut visitor, var_repr, dir);
            }
        }
        Repr::Struct(var_repr) => {
            visit_var_repr(&mut visitor, var_repr, dir);
        }
    }

    // where thing: is_trait
    fn where_predicate(thing: Type, is_trait: syn::Path) -> syn::WherePredicate {
        syn::WherePredicate::Type(syn::PredicateType {
            lifetimes: None,
            bounded_ty: thing,
            colon_token: <Token![:]>::default(),
            bounds: vec![syn::TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: is_trait,
            })]
            .into_iter()
            .collect(),
        })
    }

    // place bounds on all used type parameters and associated types
    let new_predicates = visitor
        .used
        .into_iter()
        .cloned()
        .map(|bounded_ty| where_predicate(syn::Type::Path(bounded_ty), bound.clone()));

    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .extend(new_predicates);

    generics
}

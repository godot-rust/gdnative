use std::collections::HashSet;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Generics, Ident, Type, TypePath};

// recursively visit all the field types to find what types should be bounded
pub struct BoundsVisitor<'ast> {
    all_type_params: HashSet<Ident>,
    used: HashSet<&'ast TypePath>,
}

impl<'ast> Visit<'ast> for BoundsVisitor<'ast> {
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
                self.used.insert(type_path);
            }
        }
        visit::visit_path(self, &type_path.path);
    }
}

impl<'ast> BoundsVisitor<'ast> {
    fn new(generics: &Generics) -> Self {
        let all_type_params = generics
            .type_params()
            .map(|param| param.ident.clone())
            .collect();

        BoundsVisitor {
            all_type_params,
            used: HashSet::new(),
        }
    }
}

pub fn with_visitor<'ast, F>(
    generics: Generics,
    bound: Option<&syn::Path>,
    lifetime: Option<&str>,
    op: F,
) -> Generics
where
    F: FnOnce(&mut BoundsVisitor<'ast>),
{
    let mut visitor = BoundsVisitor::new(&generics);

    op(&mut visitor);

    // where thing: is_trait
    fn where_predicate(
        thing: Type,
        bound: Option<&syn::Path>,
        lifetime: Option<&str>,
    ) -> syn::WherePredicate {
        let mut bounds = vec![];

        if let Some(bound) = bound {
            bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: bound.clone(),
            }));
        }

        if let Some(lifetime) = lifetime {
            bounds.push(syn::TypeParamBound::Lifetime(syn::Lifetime::new(
                lifetime,
                thing.span(),
            )));
        }

        syn::WherePredicate::Type(syn::PredicateType {
            lifetimes: None,
            bounded_ty: thing,
            colon_token: <Token![:]>::default(),
            bounds: bounds.into_iter().collect(),
        })
    }

    // place bounds on all used type parameters and associated types
    let mut new_predicates = visitor
        .used
        .into_iter()
        .cloned()
        .map(|bounded_ty| where_predicate(syn::Type::Path(bounded_ty), bound, lifetime))
        .collect::<Vec<_>>();

    // Add lifetime bounds to all type parameters, regardless of usage, due to how
    // lifetimes for generic types are determined.
    new_predicates.extend(generics.type_params().map(|param| {
        where_predicate(
            syn::Type::Path(syn::TypePath {
                qself: None,
                path: param.ident.clone().into(),
            }),
            None,
            lifetime,
        )
    }));

    let mut generics = generics;
    generics
        .make_where_clause()
        .predicates
        .extend(new_predicates);

    generics
}

use std::collections::HashSet;

use proc_macro2::Span;
use syn::visit::{self, Visit};
use syn::{Generics, Ident, TypePath};

pub struct Visitor {
    all_type_params: HashSet<Ident>,
    found: Vec<Span>,
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_type_path(&mut self, type_path: &'ast TypePath) {
        let path = &type_path.path;

        if let Some(seg) = path.segments.first() {
            if self.all_type_params.contains(&seg.ident) {
                // if the first segment of the type path is a known type variable, then this
                // is likely an associated type
                self.found.push(seg.ident.span());
            }
        }

        if let Some(qself) = type_path.qself.as_ref() {
            visit::visit_qself(self, qself);
        }

        visit::visit_path(self, &type_path.path);
    }
}

impl Visitor {
    fn new(generics: &Generics) -> Self {
        let all_type_params = generics
            .type_params()
            .map(|param| param.ident.clone())
            .collect();

        Visitor {
            all_type_params,
            found: Vec::new(),
        }
    }
}

pub fn with_visitor<F>(generics: &Generics, op: F) -> Vec<Span>
where
    F: FnOnce(&mut Visitor),
{
    let mut visitor = Visitor::new(generics);
    op(&mut visitor);
    visitor.found
}

use gdnative::prelude::*;

#[derive(FromVariant)]
pub struct Foo {
    // Not a 'simple' ident, has `::`
    #[variant(baz::quux)]
    bar: String,
}

fn main() {}

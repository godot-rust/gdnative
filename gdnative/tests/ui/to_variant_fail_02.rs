use gdnative::prelude::*;

#[derive(ToVariant)]
pub struct Foo {
    // Not a 'simple' ident, has `::`
    #[variant(baz::quux)]
    bar: String,
}

fn main() {}

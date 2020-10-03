use gdnative::prelude::*;

#[derive(FromVariant)]
pub struct Foo {
    // error: baz::quux is not a simple ident in the NameValue
    #[variant(baz::quux = "path::to::function")]
    bar: String,
}

fn main() {}

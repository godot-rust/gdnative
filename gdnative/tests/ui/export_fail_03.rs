use gdnative::prelude::*;

#[derive(Export, ToVariant)]
pub union Foo {
    bar: i32,
}

fn main() {}

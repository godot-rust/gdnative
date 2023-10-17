use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export(kind = "enum")]
pub union Foo {
    bar: i32,
}

fn main() {}

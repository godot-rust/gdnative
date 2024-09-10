use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export(kind = "enum")]
pub struct Foo {
    bar: i32,
}

fn main() {}

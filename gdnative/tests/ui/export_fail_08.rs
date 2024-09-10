use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export(kind = "foo")]
pub enum Foo {
    Bar,
}

fn main() {}

use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export(kind)]
pub enum Foo {
    Bar,
}

fn main() {}

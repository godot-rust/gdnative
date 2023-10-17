use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export(kind = 123)]
pub enum Foo {
    Bar,
}

fn main() {}

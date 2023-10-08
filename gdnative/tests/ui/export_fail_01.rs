use gdnative::prelude::*;

#[derive(Export, ToVariant)]
pub enum Foo {
    Bar(String),
}

fn main() {}

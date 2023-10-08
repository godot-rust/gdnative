use gdnative::prelude::*;

#[derive(Export, ToVariant)]
pub enum Foo {
    Bar(String),
    Baz { a: i32, b: u32 },
}

fn main() {}

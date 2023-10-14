use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export(kinb = "enum")]
pub enum Foo {
    Bar,
}

fn main() {}

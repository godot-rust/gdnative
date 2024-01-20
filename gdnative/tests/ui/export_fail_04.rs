use gdnative::prelude::*;

#[derive(Export, ToVariant)]
#[export]
pub enum Foo {
    Bar,
}

#[derive(Export, ToVariant)]
#[export = "foo"]
pub enum Bar {
    Foo,
}

#[derive(Export, ToVariant)]
#[export(weird format a => b)]
pub enum Baz {
    Quux,
}

fn main() {}

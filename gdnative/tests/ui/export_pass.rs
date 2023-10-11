use gdnative::prelude::*;

#[derive(Export, ToVariant, Clone, Copy)]
#[variant(enum = "repr")]
#[repr(i32)]
pub enum Foo {
    Bar,
    Baz,
}

fn main() {}

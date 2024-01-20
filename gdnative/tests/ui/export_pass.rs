use gdnative::prelude::*;

#[derive(Export, ToVariant, Clone, Copy)]
#[variant(enum = "repr")]
#[export(kind = "enum")]
#[repr(i32)]
pub enum Foo {
    Bar,
    Baz,
}

fn main() {}

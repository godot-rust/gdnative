use gdnative::prelude::*;

#[derive(FromVariant)]
// `enum` representation should only be allowed on enums
#[variant(enum = "str")]
pub struct Foo {
    bar: String,
}

#[derive(FromVariant)]
// The `str` representation should only be allowed for fieldless enums
#[variant(enum = "str")]
pub enum Bar {
    A,
    B(String),
    C,
}

fn main() {}

use gdnative::prelude::*;

#[derive(FromVariant)]
// `enum` representation should only be allowed on enums
#[variant(enum = "repr")]
pub struct Foo {
    bar: String,
}

#[derive(FromVariant)]
// The `repr` representation requires an explicit type
#[variant(enum = "repr")]
pub enum Bar {
    A,
    B,
    C,
}

#[derive(FromVariant)]
// The `repr` representation should only be allowed for fieldless enums
#[variant(enum = "repr")]
#[repr(i32)]
pub enum Baz {
    A,
    B(String),
    C,
}

fn main() {}

use gdnative::prelude::*;

#[derive(ToVariant)]
pub struct Foo {
    #[variant]
    bar: String,
}

fn main() {}

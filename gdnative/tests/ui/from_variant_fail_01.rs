use gdnative::prelude::*;

#[derive(FromVariant)]
pub struct Foo {
    #[variant]
    bar: String,
}

fn main() {}

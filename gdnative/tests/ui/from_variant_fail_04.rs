use gdnative::prelude::*;

#[derive(FromVariant)]
pub struct Foo {
    // the path to the module must be wrapped in double quotes (a string)
    #[variant(with = path)]
    bar: String,
}

fn main() {}

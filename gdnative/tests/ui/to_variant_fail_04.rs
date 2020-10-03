use gdnative::prelude::*;

#[derive(ToVariant)]
pub struct Foo {
    // the path to the module must be wrapped in double quotes (a string)
    #[variant(with = path)]
    bar: String,
}

fn main() {}

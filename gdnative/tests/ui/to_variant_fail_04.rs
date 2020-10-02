use gdnative::prelude::*;

#[derive(ToVariant)]
pub struct Foo {
    // the path to the function must be wrapped in double quotes (a string)
    #[variant(with = path)]
    bar: String,
}

fn main() {}

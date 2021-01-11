use gdnative::prelude::*;

#[derive(FromVariant)]
pub struct Foo {
    // the path to the function must be wrapped in double quotes (a string)
    #[variant(from_variant_with = path)]
    bar: String,
}

fn main() {}

use gdnative::prelude::*;

#[derive(ToVariant)]
pub struct Foo {
    // the path to the function must be wrapped in double quotes (a string)
    #[variant(to_variant_with = path)]
    bar: String,
}

fn main() {}

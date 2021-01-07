use gdnative::prelude::*;

#[derive(FromVariant)]
pub struct Foo {
    #[variant(aoeu = "aoeu")]
    bar: String,
}

fn main() {}

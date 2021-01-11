use gdnative::prelude::*;

#[derive(ToVariant)]
pub struct Foo {
    #[variant(aoeu = "aoeu")]
    bar: String,
}

fn main() {}

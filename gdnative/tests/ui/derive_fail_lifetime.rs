use gdnative::prelude::*;

#[derive(NativeClass)]
struct Foo<'a> {
    bar: &'a str,
}

impl<'a> Foo<'a> {
    fn new(_owner: &Node) -> Self {
        Foo {
            bar: "bar",
        }
    }
}

fn main() {}

use gdnative::prelude::*;

#[derive(NativeClass)]
struct Foo {}

#[methods]
impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo {}
    }
}

fn main() {}

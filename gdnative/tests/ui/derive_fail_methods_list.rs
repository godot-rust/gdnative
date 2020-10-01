use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct Foo {}

#[methods(foo, bar)]
impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo {}
    }
}

fn main() {}

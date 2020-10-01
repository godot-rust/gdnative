use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
#[user_data]
struct Foo {}

impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo {}
    }
}

fn main() {}

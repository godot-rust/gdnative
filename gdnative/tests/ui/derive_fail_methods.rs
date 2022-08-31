use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct Foo {}

impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo {}
    }

    #[method]
    fn draw(&self) {}
}

fn main() {}

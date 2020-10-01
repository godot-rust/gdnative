use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct Foo {}

impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo {}
    }

    #[export]
    fn draw(&self, _owner: &Node) {}
}

fn main() {}

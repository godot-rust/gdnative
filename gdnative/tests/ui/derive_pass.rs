use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct Foo {}

#[methods]
impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo {}
    }

    #[method]
    fn bar(&self) {}
}

fn main() {}

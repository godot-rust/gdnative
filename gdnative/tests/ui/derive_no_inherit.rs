use gdnative::prelude::*;

#[derive(NativeClass)]
struct Foo {}

#[methods]
impl Foo {
    fn new(_owner: &Reference) -> Self {
        Foo {}
    }
}

fn main() {}

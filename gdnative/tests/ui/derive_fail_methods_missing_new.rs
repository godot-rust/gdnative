use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct Foo {}

#[methods]
impl Foo {}

fn main() {}

use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
struct Foo {}

#[methods]
impl Foo {
    fn new(_base: &Node) -> Self {
        Foo {}
    }

    #[method]
    async fn optional(#[opt] self, #[base] #[opt] _base: &Node, #[async_ctx] #[opt] ctx: ()) {}

    #[method]
    fn based(#[base] self, #[base] _base: &Node, #[base] #[base] _basil: &Node, #[base] #[base] #[base] _basin: &Node) {}

    #[method]
    fn sync(self, #[async_ctx] ctx: ()) {}
}

fn main() {}

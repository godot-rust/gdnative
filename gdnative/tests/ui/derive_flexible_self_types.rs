use std::sync::Arc;

use gdnative::prelude::*;

#[derive(NativeClass)]
#[user_data(gdnative::export::user_data::ArcData<Self>)]
struct Foo {}

#[methods]
impl Foo {
    fn new(_owner: &Reference) -> Self {
        Foo {}
    }

    #[method]
    fn none() {}

    #[method]
    fn arc(self: Arc<Self>) {}

    #[method]
    fn instance(#[self] this: Instance<Self>) {}

    #[method]
    fn t_instance(#[self] this: TInstance<Self>) {}
}

fn main() {}

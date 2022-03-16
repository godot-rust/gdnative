use gdnative::export::hint::*;
use gdnative::prelude::*;

fn test_hint() -> StringHint {
    StringHint::File(EnumHint::new(vec![]))
}

#[derive(Default, NativeClass)]
#[inherit(Node)]
struct Foo {
    #[property]
    bar: String,

    // hint
    #[property(hint = "test_hint")]
    prop_hint: String,
}

#[methods]
impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo::default()
    }
}

fn main() {}

use gdnative::nativescript::export::property::*;
use gdnative::prelude::*;

fn test_hint() -> StringHint {
    StringHint::File(EnumHint::new(vec![]))
}
fn test_before_get(_this: &Foo, _owner: TRef<Node>) {}
fn test_before_set(_this: &mut Foo, _owner: TRef<Node>) {}
fn test_after_get(_this: &Foo, _owner: TRef<Node>) {}
fn test_after_set(_this: &mut Foo, _owner: TRef<Node>) {}

#[derive(Default, NativeClass)]
#[inherit(Node)]
struct Foo {
    #[property]
    bar: String,

    // hint
    #[property(hint = "test_hint")]
    prop_hint: String,

    // before get & set
    #[property(before_get = "test_before_get")]
    prop_before_get: String,
    #[property(before_set = "test_before_set")]
    prop_before_set: String,

    // after get & set
    #[property(after_get = "test_after_get")]
    prop_after_get: String,
    #[property(after_set = "test_after_set")]
    prop_after_set: String,
}

#[methods]
impl Foo {
    fn new(_owner: &Node) -> Self {
        Foo::default()
    }
}

fn main() {}

use std::{marker::PhantomData, ops::Add};

use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    // Relevant tests in GDScript
    true
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<IntOps>();
    handle.add_class::<StrOps>();
}

#[derive(NativeClass)]
struct GenericOps<T> {
    _marker: PhantomData<T>,
}

impl<T> GenericOps<T> {
    fn new(_base: &Reference) -> Self {
        GenericOps {
            _marker: PhantomData,
        }
    }
}

#[methods]
impl<T> GenericOps<T>
where
    T: FromVariant + ToVariant + Add<Output = T> + 'static,
{
    #[method]
    fn add(&self, a: T, b: T) -> T {
        a + b
    }
}

#[gdnative::derive::monomorphize]
type IntOps = GenericOps<i32>;

#[gdnative::derive::monomorphize]
type StrOps = GenericOps<GodotString>;

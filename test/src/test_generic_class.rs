use std::{marker::PhantomData, ops::Add, ops::Sub};

use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    // Relevant tests in GDScript
    true
}

#[cfg(not(feature = "no-manual-register"))]
pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<IntOps>();
    handle.add_class::<StrOps>();
}

#[cfg(feature = "no-manual-register")]
pub(crate) fn register(_handle: InitHandle) {}

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

#[methods(mixin = "SubMixin")]
impl<T> GenericOps<T>
where
    T: FromVariant + ToVariant + Sub<Output = T> + 'static,
{
    #[method]
    fn sub(&self, a: T, b: T) -> T {
        a - b
    }
}

#[gdnative::derive::monomorphize]
#[register_with(register_sub)]
type IntOps = GenericOps<i32>;

#[gdnative::derive::monomorphize]
type StrOps = GenericOps<GodotString>;

fn register_sub<T>(builder: &ClassBuilder<GenericOps<T>>)
where
    T: FromVariant + ToVariant + Sub<Output = T> + 'static,
{
    builder.mixin::<SubMixin>();
}

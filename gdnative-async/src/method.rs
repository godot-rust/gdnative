use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use futures_task::{LocalFutureObj, LocalSpawn, SpawnError};

use gdnative_core::core_types::{ToVariant, Variant};
use gdnative_core::export::{Method, NativeClass, Varargs};
use gdnative_core::log::{self, Site};
use gdnative_core::object::ownership::Shared;
use gdnative_core::object::TInstance;

use crate::rt::Context;

/// Trait for async methods. When exported, such methods return `FunctionState`-like
/// objects that can be manually resumed or yielded to completion.
///
/// Async methods are always spawned locally on the thread where they were created,
/// and never sent to another thread. This is so that we can ensure the safety of
/// emitting signals from the `FunctionState`-like object. If you need to off-load
/// some task to another thread, consider using something like
/// `futures::future::Remote` to spawn it remotely on a thread pool.
pub trait AsyncMethod<C: NativeClass>: Send + Sync + 'static {
    /// Spawns the future for result of this method with `spawner`. This is done so
    /// that implementors of this trait do not have to name their future types.
    ///
    /// If the `spawner` object is not used, the Godot side of the call will fail, output an
    /// error, and return a `Nil` variant.
    fn spawn_with(&self, spawner: Spawner<'_, C>);

    /// Returns an optional site where this method is defined. Used for logging errors in FFI wrappers.
    ///
    /// Default implementation returns `None`.
    #[inline]
    fn site() -> Option<Site<'static>> {
        None
    }
}

/// A helper structure for working around naming future types. See [`Spawner::spawn`].
pub struct Spawner<'a, C: NativeClass> {
    sp: &'static dyn LocalSpawn,
    ctx: Context,
    this: TInstance<'a, C, Shared>,
    args: Varargs<'a>,
    result: &'a mut Option<Result<(), SpawnError>>,
    /// Remove Send and Sync
    _marker: PhantomData<*const ()>,
}

impl<'a, C: NativeClass> Spawner<'a, C> {
    /// Consumes this `Spawner` and spawns a future returned by the closure. This indirection
    /// is necessary so that implementors of the `AsyncMethod` trait do not have to name their
    /// future types.
    pub fn spawn<F, R>(self, f: F)
    where
        F: FnOnce(Arc<Context>, TInstance<'_, C, Shared>, Varargs<'_>) -> R,
        R: Future<Output = Variant> + 'static,
    {
        let ctx = Arc::new(self.ctx);
        let future = f(Arc::clone(&ctx), self.this, self.args);
        *self.result = Some(
            self.sp
                .spawn_local_obj(LocalFutureObj::new(Box::new(async move {
                    let value = future.await;
                    ctx.resolve(value);
                }))),
        );
    }
}

/// Adapter for async methods that implements `Method` and can be registered.
#[derive(Clone, Copy, Default, Debug)]
pub struct Async<F> {
    f: F,
}

impl<F> Async<F> {
    /// Wrap `f` in an adapter that implements `Method`.
    #[inline]
    pub fn new(f: F) -> Self {
        Async { f }
    }
}

impl<C: NativeClass, F: AsyncMethod<C>> Method<C> for Async<F> {
    fn call(&self, this: TInstance<'_, C, Shared>, args: Varargs<'_>) -> Variant {
        if let Some(sp) = crate::executor::local_spawn() {
            let ctx = Context::new();
            let func_state = ctx.func_state();

            let mut result = None;
            self.f.spawn_with(Spawner {
                sp,
                ctx,
                this,
                args,
                result: &mut result,
                _marker: PhantomData,
            });

            match result {
                Some(Ok(())) => func_state.to_variant(),
                Some(Err(err)) => {
                    log::error(
                        Self::site().unwrap_or_default(),
                        format_args!("unable to spawn future: {}", err),
                    );
                    Variant::new()
                }
                None => {
                    log::error(
                        Self::site().unwrap_or_default(),
                        format_args!("implementation did not spawn a future"),
                    );
                    Variant::new()
                }
            }
        } else {
            log::error(
                Self::site().unwrap_or_default(),
                "a global executor must be set before any async methods can be called on this thread",
            );
            Variant::new()
        }
    }
}

use std::marker::PhantomData;

use gdnative_bindings::Object;
use gdnative_core::object::SubClass;

use gdnative_core::core_types::{GodotError, Variant};
use gdnative_core::export::{Instance, RefInstance};
use gdnative_core::init::InitHandle;
use gdnative_core::object::ownership::Shared;
use gdnative_core::object::TRef;

use crate::future;

mod bridge;
mod func_state;

use func_state::FuncState;

/// Context for creating `yield`-like futures in async methods.
pub struct Context {
    func_state: Instance<FuncState, Shared>,
    /// Remove Send and Sync
    _marker: PhantomData<*const ()>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Context {
            func_state: FuncState::new().into_shared(),
            _marker: PhantomData,
        }
    }

    pub(crate) fn func_state(&self) -> Instance<FuncState, Shared> {
        self.func_state.clone()
    }

    fn safe_func_state(&self) -> RefInstance<'_, FuncState, Shared> {
        // SAFETY: FuncState objects are bound to their origin threads in Rust, and
        // Context is !Send, so this is safe to call within this type.
        // Non-Rust code is expected to be following the official guidelines as per
        // the global safety assumptions. Since a reference of `FuncState` is held by
        // Rust, it voids the assumption to send the reference to any thread aside from
        // the one where it's created.
        unsafe { self.func_state.assume_safe() }
    }

    pub(crate) fn resolve(&self, value: Variant) {
        func_state::resolve(self.safe_func_state(), value);
    }

    /// Returns a future that waits until the corresponding `FunctionState` object
    /// is manually resumed from GDScript, and yields the argument to `resume` or `Nil`
    /// if nothing is passed.
    ///
    /// Calling this function will put the associated `FunctionState`-like object in
    /// resumable state, and will make it emit a `resumable` signal if it isn't in that
    /// state already.
    ///
    /// Only the most recent future created from this `Context` is guaranteed to resolve
    /// upon a `resume` call. If any previous futures weren't `await`ed to completion, they
    /// are no longer guaranteed to resolve, and have unspecified, but safe behavior
    /// when polled.
    pub fn until_resume(&self) -> future::Yield<Variant> {
        let (future, resume) = future::make();
        func_state::make_resumable(self.safe_func_state(), resume);
        future
    }

    /// Returns a future that waits until the specified signal is emitted, if connection succeeds.
    /// Yields any arguments emitted with the signal.
    ///
    /// Only the most recent future created from this `Context` is guaranteed to resolve
    /// when the signal is emitted. If any previous futures weren't `await`ed to completion, they
    /// are no longer guaranteed to resolve, and have unspecified, but safe behavior
    /// when polled.
    ///
    /// # Errors
    ///
    /// If connection to the signal failed.
    pub fn signal<C>(
        &self,
        obj: TRef<'_, C>,
        signal: &str,
    ) -> Result<future::Yield<Vec<Variant>>, GodotError>
    where
        C: SubClass<Object>,
    {
        let (future, resume) = future::make();
        bridge::SignalBridge::connect(obj.upcast(), signal, resume)?;
        Ok(future)
    }
}

/// Adds required supporting NativeScript classes to `handle`. This must be called once and
/// only once per initialization.
pub fn register_runtime(handle: &InitHandle) {
    handle.add_class::<bridge::SignalBridge>();
    handle.add_class::<func_state::FuncState>();
}

/// Releases all observers still in use. This should be called in the
/// `godot_gdnative_terminate` callback.
pub fn terminate_runtime() {
    bridge::terminate();
}

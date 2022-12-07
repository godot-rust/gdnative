use std::fmt::Display;
use std::marker::PhantomData;

use func_state::FuncState;
use gdnative_bindings::Object;
use gdnative_core::core_types::{GodotError, Variant};
use gdnative_core::init::InitHandle;
use gdnative_core::object::{Instance, SubClass, TInstance, TRef};

use crate::future;

mod bridge;
mod func_state;

/// Context for creating `yield`-like futures in async methods.
pub struct Context {
    func_state: Instance<FuncState>,
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

    pub(crate) fn func_state(&self) -> Instance<FuncState> {
        self.func_state.clone()
    }

    fn safe_func_state(&self) -> TInstance<'_, FuncState> {
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
///
/// This registers the internal types under an unspecified prefix, with the intention to avoid
/// collision with user types. Users may provide a custom prefix using
/// [`register_runtime_with_prefix`], should it be necessary to name these types.
pub fn register_runtime(handle: &InitHandle) {
    register_runtime_with_prefix(handle, "__GDNATIVE_ASYNC_INTERNAL__")
}

/// Adds required supporting NativeScript classes to `handle`. This must be called once and
/// only once per initialization.
///
/// The user should ensure that no other NativeScript types is registered under the
/// provided prefix.
pub fn register_runtime_with_prefix<S>(handle: &InitHandle, prefix: S)
where
    S: Display,
{
    handle.add_class_as::<bridge::SignalBridge>(format!("{prefix}SignalBridge"));
    handle.add_class_as::<func_state::FuncState>(format!("{prefix}FuncState"));
}

/// Releases all observers still in use. This should be called in the
/// `godot_gdnative_terminate` callback.
pub fn terminate_runtime() {
    bridge::terminate();
}

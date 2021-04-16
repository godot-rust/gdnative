use futures_task::LocalSpawn;
use once_cell::unsync::OnceCell as UnsyncCell;
use thiserror::Error;

thread_local!(
    static LOCAL_SPAWN: UnsyncCell<&'static dyn LocalSpawn> = UnsyncCell::new();
);

/// Error returned by `set_*_executor` if an executor of the kind has already been set.
#[derive(Error, Debug)]
#[error("an executor is already set")]
pub struct SetExecutorError {
    _private: (),
}

impl SetExecutorError {
    fn new() -> Self {
        SetExecutorError { _private: () }
    }
}

pub(crate) fn local_spawn() -> Option<&'static dyn LocalSpawn> {
    LOCAL_SPAWN.with(|cell| cell.get().copied())
}

/// Sets the global executor for the current thread to a `Box<dyn LocalSpawn>`. This value is leaked.
pub fn set_boxed_executor(sp: Box<dyn LocalSpawn>) -> Result<(), SetExecutorError> {
    set_executor(Box::leak(sp))
}

/// Sets the global executor for the current thread to a `&'static dyn LocalSpawn`.
pub fn set_executor(sp: &'static dyn LocalSpawn) -> Result<(), SetExecutorError> {
    LOCAL_SPAWN.with(|cell| cell.set(sp).map_err(|_| SetExecutorError::new()))
}

/// Sets the global executor for the current thread with a function that will only be called
/// if an executor isn't set yet.
pub fn ensure_executor_with<F>(f: F)
where
    F: FnOnce() -> &'static dyn LocalSpawn,
{
    LOCAL_SPAWN.with(|cell| {
        cell.get_or_init(f);
    });
}

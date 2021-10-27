use std::cell::Cell;

use futures_task::LocalSpawn;

thread_local!(
    static LOCAL_SPAWN: Cell<Option<&'static dyn LocalSpawn>> = Cell::new(None);
);

pub(crate) fn local_spawn() -> Option<&'static dyn LocalSpawn> {
    LOCAL_SPAWN.with(|cell| cell.get())
}

/// Sets the global executor for the current thread to a `Box<dyn LocalSpawn>`. This value is leaked.
pub fn set_boxed_executor(sp: Box<dyn LocalSpawn>) {
    set_executor(Box::leak(sp))
}

/// Sets the global executor for the current thread to a `&'static dyn LocalSpawn`.
pub fn set_executor(sp: &'static dyn LocalSpawn) {
    LOCAL_SPAWN.with(|cell| cell.set(Some(sp)))
}

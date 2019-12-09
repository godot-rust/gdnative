use specs::prelude::*;
use std::sync::{Mutex, MutexGuard, TryLockError};
use your_state::YourState;

lazy_static! {
    static ref GLOBAL_STATE: Mutex<Box<YourState>> =
        Mutex::new(Box::<YourState>::new(YourState::new()));
}

pub fn get_singleton_state() -> MutexGuard<'static, Box<YourState>> {
    match GLOBAL_STATE.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            panic!("Global sector state was poisoned!");
        }
    }
}

pub fn try_get_singleton_state() -> Option<MutexGuard<'static, Box<YourState>>> {
    match GLOBAL_STATE.try_lock() {
        Ok(guard) => Some(guard),
        Err(TryLockError::WouldBlock) => None,
        Err(TryLockError::Poisoned(_err)) => {
            panic!("Global sector state was poisoned!");
        }
    }
}

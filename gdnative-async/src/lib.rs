//! Runtime async support for godot-rust.
//!
//! This crate contains types and functions that enable using async code with godot-rust.
//!
//! # Safety assumptions
//!
//! This crate assumes that all user non-Rust code follow the official threading guidelines.

// Workaround for macros that expect the `gdnative` crate.
extern crate gdnative_core as gdnative;

mod executor;
mod future;
mod method;
mod rt;

pub use executor::{ensure_executor_with, set_boxed_executor, set_executor, SetExecutorError};
pub use method::{Async, AsyncMethod, Spawner};
pub use rt::{register_runtime, terminate_runtime, Context, InitError};

//! Functionality for user-defined types exported to the engine (native scripts).
//!
//! NativeScript allows users to have their own scripts in a native language (in this case Rust).
//! It is _not_ the same as GDNative, the native interface to call into Godot.
//! Symbols in this module allow registration, exporting and management of user-defined types
//! which are wrapped in native scripts.
//!
//! If you are looking for how to manage Godot core types or classes (objects), check
//! out the [`core_types`][crate::core_types] and [`object`][crate::object] modules, respectively.
//!
//! To handle initialization and shutdown of godot-rust, take a look at the [`init`][crate::init] module.
//!
//! For full examples, see [`examples`](https://github.com/godot-rust/godot-rust/tree/master/examples)
//! in the godot-rust repository.

mod class;
mod class_builder;
mod macros;
mod method;
mod property;
mod signal;

pub(crate) mod class_registry;
pub(crate) mod emplace;
pub(crate) mod type_tag;

pub mod user_data;

pub use crate::godot_wrap_method;
pub use class::*;
pub use class_builder::*;
pub use method::*;
pub use property::*;
pub use signal::*;

use std::fmt;

/// Error to panic in 'catch_unwind()'.
/// PanickedError displays the contents if the payload passed during panic has type &str or String.
#[derive(Debug)]
pub struct PanickedError {
    cause: Box<dyn std::any::Any + Send + 'static>,
}

impl std::error::Error for PanickedError {}
impl fmt::Display for PanickedError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cause = &self.cause;
        match cause.downcast_ref::<&str>() {
            Some(s) => write!(f, "{}", s),
            None => match cause.downcast_ref::<String>() {
                Some(s) => write!(f, "{}", s),
                None => write!(f, "unknown panicked"),
            },
        }
    }
}

impl PanickedError {
    // Returns the payload associated with the panic.
    #[inline]
    pub fn payload(&self) -> &(dyn std::any::Any + Send + 'static) {
        &self.cause
    }
}

/// Wrapper function for `std:panic:catch_unwind()` that returns `PanickedError` when panicked.
/// See also: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
#[inline]
pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> Result<R, PanickedError> {
    let result = std::panic::catch_unwind(f);
    result.map_err(|cause| PanickedError { cause })
}

// For silenced lints/warnings, see also gdnative-sys/src/lib.rs

// Generated bindings don't follow some conventions
#![allow(non_snake_case)]
#![allow(unused_unsafe)]
// False positives on generated drops that enforce lifetime
#![allow(clippy::drop_copy)]
// False positives on thread-safe singletons
#![allow(clippy::non_send_fields_in_send_ty)]
// Disable non-critical lints for generated code
#![allow(clippy::style, clippy::complexity, clippy::perf)]

mod generated;
pub use generated::*;

pub mod utils;

pub(crate) mod icalls;

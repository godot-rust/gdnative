#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_unsafe)]
// False positives on generated drops that enforce lifetime
#![allow(clippy::drop_copy)]
// Disable non-critical lints for generated code.
#![allow(clippy::style, clippy::complexity, clippy::perf)]

mod generated;
pub use generated::*;

pub mod utils;

pub(crate) mod icalls;

#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]
#![allow(unused_unsafe)]
// False positives on generated drops that enforce lifetime
#![allow(clippy::drop_copy)]
// Disable non-critical lints for generated code.
#![allow(clippy::style, clippy::complexity, clippy::perf)]

pub use gdnative_core::*;

use crate::private::get_api;
use crate::sys;

use libc;
use std::ops::*;
use std::sync::Once;

include!(concat!(env!("OUT_DIR"), "/bindings_types.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings_traits.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings_methods.rs"));

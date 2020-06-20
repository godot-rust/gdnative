#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]
#![allow(unused_unsafe)]
// False positives on generated drops that enforce lifetime
#![allow(clippy::drop_copy)]
// Disable non-critical lints for generated code.
#![allow(clippy::style, clippy::complexity, clippy::perf)]

use gdnative_core::object::{self, PersistentRef};
use gdnative_core::private::get_api;
use gdnative_core::sys;
use gdnative_core::sys::GodotApi;
use gdnative_core::*;

use libc;
use std::ops::*;
use std::sync::Once;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

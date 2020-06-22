#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]
#![allow(unused_unsafe)]
// False positives on generated drops that enforce lifetime
#![allow(clippy::drop_copy)]
// Disable non-critical lints for generated code.
#![allow(clippy::style, clippy::complexity, clippy::perf)]

use super::*;
use crate::object::{self};
use crate::private::get_api;
use crate::ref_kind;
use crate::sys;
use crate::sys::GodotApi;
use crate::thread_access;

use libc;
use std::ops::*;
use std::sync::Once;

use std::os::raw::c_char;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

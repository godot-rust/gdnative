#![allow(non_snake_case)] // because of the generated bindings.

extern crate gdnative_core;
pub extern crate libc;

pub use gdnative_core::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/common_types.rs"));

#![allow(non_snake_case)] // because of the generated bindings.

extern crate gdnative_core;

use gdnative_core::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use std::mem;
use libc;

include!(concat!(env!("OUT_DIR"), "/graphics_types.rs"));

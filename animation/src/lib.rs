#![allow(non_snake_case)] // because of the generated bindings.

extern crate gdnative_core;

use gdnative_core::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;

include!(concat!(env!("OUT_DIR"), "/animation_types.rs"));

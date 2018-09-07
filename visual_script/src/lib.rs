#![allow(non_snake_case)] // because of the generated bindings.

pub extern crate gdnative_core;

pub use gdnative_core as core;
use core::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use std::mem;
use libc;

include!(concat!(env!("OUT_DIR"), "/visual_script_types.rs"));

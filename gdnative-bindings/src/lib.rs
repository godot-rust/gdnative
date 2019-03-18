#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]

pub use gdnative_core::*;

use crate::sys;
use crate::get_api;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
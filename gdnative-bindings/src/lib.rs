#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]

pub use gdnative_core::*;

use crate::get_api;
use crate::sys;

use libc;
use std::ops::*;
use std::sync::{Once, ONCE_INIT};

include!(concat!(env!("OUT_DIR"), "/bindings_types.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings_traits.rs"));
include!(concat!(env!("OUT_DIR"), "/bindings_methods.rs"));

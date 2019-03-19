#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]


use crate::sys;
use crate::get_api;
use super::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;

use std::ptr;
use std::os::raw::c_char;

include!(concat!(env!("OUT_DIR"), "/core_types.rs"));
include!(concat!(env!("OUT_DIR"), "/core_traits.rs"));
include!(concat!(env!("OUT_DIR"), "/core_methods.rs"));

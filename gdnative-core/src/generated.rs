#![allow(non_snake_case)] // because of the generated bindings.
#![allow(unused_imports)]

use super::*;
use crate::get_api;
use crate::sys;
use crate::sys::GodotApi;

use libc;
use std::ops::*;
use std::sync::Once;

use std::os::raw::c_char;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/core_types.rs"));
include!(concat!(env!("OUT_DIR"), "/core_traits.rs"));
include!(concat!(env!("OUT_DIR"), "/core_methods.rs"));

#![allow(unused_variables)]

#[cfg(feature = "ptrcall")]
use libc;

use libc::c_char;
use std::mem;
use std::ptr;

use gdnative_core::core_types::*;
use gdnative_core::object::*;
use gdnative_core::object::{memory, ownership};
use gdnative_core::private::get_api;
use gdnative_core::sys;
use gdnative_core::sys::GodotApi;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

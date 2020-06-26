use libc;
use std::sync::Once;

use gdnative_core::object::*;
use gdnative_core::thread_access;
use gdnative_core::*;

use gdnative_core::private::get_api;
use gdnative_core::sys;
use gdnative_core::sys::GodotApi;
use gdnative_core::vector3;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

use libc;
use std::ops::*;
use std::sync::Once;

use gdnative_core::generated::object;
use gdnative_core::generated::reference;
use gdnative_core::generated::reference::*;
use gdnative_core::private::get_api;
use gdnative_core::*;
pub use gdnative_core::RefCounted;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

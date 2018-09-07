#![allow(non_snake_case)] // because of the generated bindings.

pub extern crate gdnative_core;
pub extern crate gdnative_video;
pub use gdnative_core as core;
pub use gdnative_video as video;

use core::*;
use video::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use std::mem;
use libc;

include!(concat!(env!("OUT_DIR"), "/ui_types.rs"));

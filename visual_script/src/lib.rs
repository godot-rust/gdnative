#![allow(non_snake_case)] // because of the generated bindings.

pub extern crate gdnative_common;

pub use gdnative_common as common;
use common::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/visual_script_types.rs"));

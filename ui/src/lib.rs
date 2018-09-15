#![allow(non_snake_case)] // because of the generated bindings.

pub extern crate gdnative_common;
pub extern crate gdnative_video;
pub use gdnative_common as common;
pub use gdnative_video as video;

use common::*;
use video::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/ui_types.rs"));

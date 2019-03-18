#![allow(non_snake_case)] // because of the generated bindings.

use gdnative_common::*;
use gdnative_video::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/ui_types.rs"));

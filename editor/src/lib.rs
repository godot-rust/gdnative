#![allow(non_snake_case)] // because of the generated bindings.

extern crate gdnative_common;
extern crate gdnative_ui;
extern crate gdnative_animation;

use gdnative_common::*;
use gdnative_ui::*;
use gdnative_animation::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/editor_types.rs"));

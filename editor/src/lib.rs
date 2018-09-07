#![allow(non_snake_case)] // because of the generated bindings.

extern crate gdnative_core;
extern crate gdnative_ui;
extern crate gdnative_animation;

use gdnative_core::*;
use gdnative_ui::*;
use gdnative_animation::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;

include!(concat!(env!("OUT_DIR"), "/editor_types.rs"));

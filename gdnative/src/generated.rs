#![allow(non_snake_case)] // because of the generated bindings.

use sys;
use get_api;
use geom::*;
use {GodotRef, GodotClass, Variant, Array, Color, Rid, NodePath, GodotClassInfo};

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;


include!(concat!(env!("OUT_DIR"), "/types.rs"));


#![allow(non_snake_case)] // because of the generated bindings.

use sys;
use get_api;
use geom::*;
use {GodotRef, GodotClass, GodotClassInfo};
use {Variant, Array, Color, Rid, NodePath, Dictionary};
use {ByteArray, StringArray, Vector2Array, Vector3Array, ColorArray};

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;


include!(concat!(env!("OUT_DIR"), "/types.rs"));


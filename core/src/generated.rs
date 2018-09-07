#![allow(non_snake_case)] // because of the generated bindings.

use sys;
use get_api;
use geom::*;
use super::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;


include!(concat!(env!("OUT_DIR"), "/core_types.rs"));


#![allow(non_snake_case)] // because of the generated bindings.

use gdnative_common::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/input_types.rs"));

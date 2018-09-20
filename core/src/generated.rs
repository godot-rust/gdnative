#![allow(non_snake_case)] // because of the generated bindings.

use sys;
use get_api;
use super::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;
use libc;

include!(concat!(env!("OUT_DIR"), "/core_types.rs"));

impl ToVariant for Object {
    fn to_variant(&self) -> Variant {
        Variant::from_object(self)
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        variant.try_to_object()
    }
}

//! # Rust bindings for the Godot game engine
//!
//! This crate contains high-level wrappers around the Godot game engine's gdnative API.
//! Some of the types were automatically generated from the engine's JSON API description,
//! and some other types are hand made wrappers around the core C types.
//!
//! ## Memory management
//!
//! ### Reference counting
//!
//! A lot of the types provided by the engine are internally reference counted and
//! allow mutable aliasing.
//! In rust parlance this means that a type such as `gdnative::ConcavePolygonShape2D`
//! is functionally equivalent to a `Rc<Cell<Something>>` rather than `Rc<Something>`.
//!
//! Since it is easy to expect containers and other types to allocate a copy of their
//! content when using the `Clone` trait, most of these types do not implement `Clone`
//! and instead implement [`RefCounted`](./trait.RefCounted.html) which provides a
//! `new_ref(&self) -> Self` method to create references to the same collection or object.
//!
//! ### Manually managed objects
//!
//! Some types are manually managed. This means that ownership can be passed to the
//! engine or the object must be carefully deallocated using the object's `free`  method.
//!

#![deny(clippy::missing_inline_in_public_items)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![cfg_attr(feature = "gd_test", allow(clippy::blacklisted_name))]

#[doc(hidden)]
pub extern crate gdnative_sys as sys;
#[doc(hidden)]
pub extern crate libc;

#[cfg(feature = "gd_test")]
#[macro_use]
extern crate approx;

// Macros have to be processed before they are used.
mod macros;

pub mod core_types;
pub use core_types::*;

#[cfg(feature = "nativescript")]
pub mod nativescript;
#[cfg(feature = "nativescript")]
pub use nativescript::*;

mod generated;
#[doc(hidden)]
pub mod object;
mod ref_counted;
pub mod thread_access;

/// Internal low-level API for use by macros and generated bindings. Not a part of the public API.
#[doc(hidden)]
pub mod private;

pub use crate::generated::*;
pub use crate::object::{Free, GodotObject, Instanciable, QueueFree};
pub use crate::ref_counted::*;

pub use sys::GodotApi;

#[doc(inline)]
pub use error::GodotError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Vector3Axis {
    X = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_X as u32,
    Y = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Y as u32,
    Z = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Z as u32,
}

pub type GodotResult = Result<(), GodotError>;

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
#[macro_use]
extern crate bitflags;
extern crate parking_lot;

#[cfg(feature = "gd_test")]
#[macro_use]
extern crate approx;

pub mod geom;

mod macros;
#[macro_use]
mod class;
pub mod access;
mod byte_array;
mod color;
mod color_array;
pub mod dictionary;
pub mod error;
mod float32_array;
mod generated;
pub mod init;
mod int32_array;
mod node_path;
#[doc(hidden)]
pub mod object;
mod point2;
mod ref_counted;
mod rid;
mod string;
mod string_array;
pub mod thread_access;
mod type_tag;
pub mod typed_array;
pub mod user_data;
mod variant;
mod variant_array;
mod vector2;
mod vector2_array;
mod vector3;
mod vector3_array;

/// Internal low-level API for use by macros and generated bindings. Not a part of the public API.
#[doc(hidden)]
pub mod private;

pub use crate::byte_array::*;
pub use crate::class::*;
pub use crate::color::*;
pub use crate::color_array::*;
pub use crate::dictionary::Dictionary;
pub use crate::float32_array::*;
pub use crate::generated::*;
pub use crate::geom::*;
pub use crate::int32_array::*;
pub use crate::node_path::*;
pub use crate::object::{Free, GodotObject, Instanciable, QueueFree};
pub use crate::point2::*;
pub use crate::ref_counted::*;
pub use crate::rid::*;
pub use crate::string::*;
pub use crate::string_array::*;
pub use crate::typed_array::TypedArray;
pub use crate::user_data::Map;
pub use crate::user_data::MapMut;
pub use crate::user_data::UserData;
pub use crate::variant::*;
pub use crate::variant_array::*;
pub use crate::vector2::*;
pub use crate::vector2_array::*;
pub use crate::vector3::*;
pub use crate::vector3_array::*;

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

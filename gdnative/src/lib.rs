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
//! and instead provide a `new_ref(&self) -> Self` method to create references to the
//! same collection or object.
//!
//! ### Manually managed objects
//!
//! Some types are manually managed. This means that ownership can be passed to the
//! engine or the object must be carefully deallocated using the object's `free`  method.
//!

// TODO: document feature flags

// TODO: add logo using #![doc(html_logo_url = "https://<url>")]

// TODO: currently the generated classes are not showing in the the gdnative crate
// documentation, and are only appearing in the sub-crates. It would make the doc
// a lot easier to navigate if we could gather all classes here.

#[doc(inline)]
pub use gdnative_core::*;
#[doc(inline)]
pub use gdnative_derive::*;

#[doc(inline)]
#[cfg(feature = "bindings")]
pub use gdnative_bindings::*;

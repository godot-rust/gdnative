//! # Rust bindings for the Godot game engine
//!
//! This crate contains high-level wrappers around the core types of Godot Engine's GDNative
//! API, and the NativeScript feature which enables Rust code to be used as scripts.
//!
//! ## Memory management for core types
//!
//! Wrappers for most core types expose safe Rust interfaces, and it's unnecessary to mind
//! memory management most of the times. The exceptions are `VariantArray` and `Dictionary`,
//! internally reference-counted collections with "interior mutability" in Rust parlance. These
//! types are modelled using the "typestate" pattern to enforce that the official
//! [thread-safety guidelines][thread-safety]. For more information, read the type-level
//! documentation for these types.
//!
//! Since it is easy to expect containers and other types to allocate a copy of their
//! content when using the `Clone` trait, some types do not implement `Clone` and instead
//! implement [`NewRef`](object::NewRef) which provides a `new_ref(&self) -> Self` method
//! to create references to the same collection or object.
//!
//! [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html

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
mod init;

#[cfg(feature = "nativescript")]
pub mod nativescript;

#[doc(hidden)]
pub mod log;
pub mod object;
pub mod ref_kind;
pub mod thread_access;

/// Internal low-level API for use by macros and generated bindings. Not a part of the public API.
#[doc(hidden)]
pub mod private;

//
// Re-exports
//

pub use init::{InitializeInfo, TerminateInfo};

#[deprecated(since = "0.10", note = "use core_types::GodotResult")]
#[doc(hidden)]
pub type GodotResult = core_types::GodotResult;

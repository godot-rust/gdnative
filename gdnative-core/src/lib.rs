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
#![allow(
    clippy::transmute_ptr_to_ptr,
    clippy::missing_safety_doc,
    clippy::if_then_panic
)]
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

#[cfg(feature = "nativescript")]
pub mod nativescript;

pub mod log;
pub mod object;

/// Internal low-level API for use by macros and generated bindings. Not a part of the public API.
#[doc(hidden)]
pub mod private;

use core_types::GodotString;

/// Context for the [`godot_gdnative_terminate`] callback.
pub struct TerminateInfo {
    in_editor: bool,
}

impl TerminateInfo {
    #[inline]
    #[doc(hidden)] // avoids clippy warning: unsafe function's docs miss `# Safety` section
    pub unsafe fn new(options: *mut crate::sys::godot_gdnative_terminate_options) -> Self {
        assert!(!options.is_null(), "options were NULL");

        let crate::sys::godot_gdnative_terminate_options { in_editor } = *options;

        Self { in_editor }
    }

    /// Returns `true` if the library is loaded in the Godot Editor.
    #[inline]
    pub fn in_editor(&self) -> bool {
        self.in_editor
    }
}

/// Context for the [`godot_gdnative_init`] callback.
pub struct InitializeInfo {
    in_editor: bool,
    active_library_path: GodotString,
    options: *mut crate::sys::godot_gdnative_init_options,
}

impl InitializeInfo {
    /// Returns true if the library is loaded in the Godot Editor.
    #[inline]
    pub fn in_editor(&self) -> bool {
        self.in_editor
    }

    /// Returns a path to the library relative to the project.
    ///
    /// Example: `res://../../target/debug/libhello_world.dylib`
    #[inline]
    pub fn active_library_path(&self) -> &GodotString {
        &self.active_library_path
    }

    /// Internal interface.
    ///
    /// # Safety
    ///
    /// Will `panic!()` if options is NULL, UB if invalid.
    #[inline]
    #[doc(hidden)]
    pub unsafe fn new(options: *mut crate::sys::godot_gdnative_init_options) -> Self {
        assert!(!options.is_null(), "options were NULL");
        let crate::sys::godot_gdnative_init_options {
            in_editor,
            active_library_path,
            ..
        } = *options;

        let active_library_path = GodotString::clone_from_sys(*active_library_path);

        Self {
            in_editor,
            active_library_path,
            options,
        }
    }

    #[inline]
    pub fn report_loading_error<T>(&self, message: T)
    where
        T: std::fmt::Display,
    {
        let crate::sys::godot_gdnative_init_options {
            report_loading_error,
            gd_native_library,
            ..
        } = unsafe { *self.options };

        if let Some(report_loading_error_fn) = report_loading_error {
            // Add the trailing zero and convert Display => String
            let message = format!("{}\0", message);

            // Convert to FFI compatible string
            let message = std::ffi::CStr::from_bytes_with_nul(message.as_bytes())
                .expect("message should not have a NULL");

            unsafe {
                report_loading_error_fn(gd_native_library, message.as_ptr());
            }
        }
    }
}

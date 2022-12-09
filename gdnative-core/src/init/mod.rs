//! Global initialization and termination of the library.
//!
//! This module provides all the plumbing required for global initialization and shutdown of godot-rust.
//!
//! ## Init and exit hooks
//!
//! Three endpoints are automatically invoked by the engine during startup and shutdown:
//!
//! * [`godot_gdnative_init`],
//! * [`godot_nativescript_init`],
//! * [`godot_gdnative_terminate`],
//!
//! All three must be present. To quickly define all three endpoints using the default names,
//! use [`godot_init`].
//!
//! ## Registering script classes
//!
//! [`InitHandle`] is the registry of all your exported symbols.
//! To register script classes, call [`InitHandle::add_class()`] or [`InitHandle::add_tool_class()`]
//! in your `godot_nativescript_init` or `godot_init` callback:
//!
//! ```no_run
//! use gdnative::prelude::*;
//!
//! #[derive(NativeClass)]
//! # #[no_constructor]
//! struct HelloWorld { /* ... */ }
//!
//! #[methods]
//! impl HelloWorld { /* ... */ }
//!
//! fn init(handle: InitHandle) {
//!     handle.add_class::<HelloWorld>();
//! }
//!
//! godot_init!(init);
//! ```

mod info;
mod init_handle;
mod macros;

pub mod diagnostics;

pub use info::*;
pub use init_handle::*;

bitflags::bitflags! {
    /// Initialization level used to distinguish the source of init actions, such as class registration.
    /// Internal API.
    #[doc(hidden)]
    pub struct InitLevel: u8 {
        /// Init level for automatic registration
        const AUTO = 1;
        /// Init level for user code
        const USER = 2;
    }
}

#[doc(hidden)]
#[cfg(feature = "inventory")]
#[inline]
pub fn auto_register(init_handle: InitHandle) {
    for plugin in inventory::iter::<crate::private::AutoInitPlugin> {
        (plugin.f)(init_handle);
    }
}

#[doc(hidden)]
#[cfg(not(feature = "inventory"))]
#[inline]
pub fn auto_register(_init_handle: InitHandle) {
    // Nothing to do here.
}

pub use crate::{
    godot_gdnative_init, godot_gdnative_terminate, godot_init, godot_nativescript_init,
};

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

pub use info::*;
pub use init_handle::*;

pub use crate::{
    godot_gdnative_init, godot_gdnative_terminate, godot_init, godot_nativescript_init,
};

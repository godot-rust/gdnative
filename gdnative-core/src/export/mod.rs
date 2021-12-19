//! Functionality for user-defined types exported to the engine (native scripts)
//!
//! NativeScript allows users to have their own scripts in a native language (in this case Rust).
//! It is _not_ the same as GDNative, the native interface to call into Godot.
//! Symbols in this module allow registration, exporting and management of user-defined types
//! which are wrapped in native scripts.
//!
//! If you are looking for how to manage Godot core types or classes (objects), check
//! out the [`core_types`][crate::core_types] and [`object`][crate::object] modules, respectively.
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
//! in your [`godot_nativescript_init`] or [`godot_init`] callback:
//!
//! ```ignore
//! use gdnative::prelude::*;
//!
//! fn init(handle: InitHandle) {
//!     handle.add_class::<HelloWorld>();
//! }
//!
//! godot_init!(init);
//! ```
//!
//! For full examples, see [`examples`](https://github.com/godot-rust/godot-rust/tree/master/examples)
//! in the godot-rust repository.

mod class;
mod class_builder;
mod macros;
mod method;
mod property;
mod signal;

pub(crate) mod class_registry;
pub(crate) mod emplace;
pub(crate) mod type_tag;

pub mod user_data;

pub use crate::godot_wrap_method;
pub use class::*;
pub use class_builder::*;
pub use method::*;
pub use property::*;
pub use signal::*;

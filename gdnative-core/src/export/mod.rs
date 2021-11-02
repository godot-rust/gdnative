//! Functionality for user-defined types exported to the engine (native scripts)
//!
//! NativeScript allows users to have their own scripts in a native language (in this case Rust).
//! It is _not_ the same as GDNative, the native interface to call into Godot.
//!
//! Symbols in this module allow registration, exporting and management of user-defined types
//! which are wrapped in native scripts.
//!
//! If you are looking for how to manage Godot core types or classes (objects), check
//! out the [`core_types`][crate::core_types] and [`object`][crate::object] modules, respectively.

mod class;
mod class_builder;
mod emplace;
mod macros;
mod method;
mod property;

pub(crate) mod class_registry;
pub(crate) mod type_tag;

pub use class::*;
pub use method::*;
pub use property::*;

pub mod profiler;
pub mod user_data;

pub use crate::godot_wrap_method;
pub use class::*;
pub use class_builder::*;
pub use method::*;
pub use property::*;

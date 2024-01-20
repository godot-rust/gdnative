//! Functionality for user-defined types exported to the engine (native scripts).
//!
//! NativeScript allows users to have their own scripts in a native language (in this case Rust).
//! It is _not_ the same as GDNative, the native interface to call into Godot.
//! Symbols in this module allow registration, exporting and management of user-defined types
//! which are wrapped in native scripts.
//!
//! If you are looking for how to manage Godot core types or classes (objects), check
//! out the [`core_types`][crate::core_types] and [`object`][crate::object] modules, respectively.
//!
//! To handle initialization and shutdown of godot-rust, take a look at the [`init`][crate::init] module.
//!
//! For full examples, see [`examples`](https://github.com/godot-rust/godot-rust/tree/master/examples)
//! in the godot-rust repository.

mod class;
mod class_builder;
mod method;
mod property;
mod signal;

pub(crate) mod class_registry;
pub(crate) mod emplace;
pub(crate) mod type_tag;

pub mod user_data;

pub use class::*;
pub use class_builder::*;
#[doc(inline)]
pub use gdnative_derive::godot_wrap_method;
pub use method::*;
pub use property::*;
pub use signal::*;

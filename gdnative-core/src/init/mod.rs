//! Global initialization and termination of the library.

mod info;
mod init_handle;
mod macros;

pub use info::*;
pub use init_handle::*;

pub use crate::{
    godot_gdnative_init, godot_gdnative_terminate, godot_init, godot_nativescript_init,
};

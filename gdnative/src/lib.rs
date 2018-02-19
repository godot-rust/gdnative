#[doc(hidden)]
pub extern crate libc;
#[doc(hidden)]
pub extern crate gdnative_sys as sys;
#[macro_use]
extern crate bitflags;

pub extern crate gdnative_geom as geom;

mod macros;
#[macro_use]
mod class;
mod internal;
mod property;
mod godot_type;
mod color;
mod variant;
mod array;
mod dictionary;
mod rid;
mod generated;
mod node_path;
mod string;

pub use internal::*;
pub use property::*;
pub use class::*;
pub use godot_type::*;
pub use variant::*;
pub use array::*;
pub use dictionary::*;
pub use geom::*;
pub use color::*;
pub use rid::*;
pub use node_path::*;
pub use generated::*;
pub use string::*;

#[doc(hidden)]
pub static mut GODOT_API: Option<GodotApi> = None;
#[inline]
#[doc(hidden)]
pub fn get_api() -> &'static GodotApi {
    unsafe {
        &GODOT_API.as_ref()
            .expect("API not bound")
    }
}


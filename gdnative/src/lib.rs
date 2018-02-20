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
mod variant_array;
mod dictionary;
mod rid;
mod generated;
mod node_path;
mod string;
mod byte_array;
mod string_array;
mod vector2_array;
mod vector3_array;
mod color_array;

pub use internal::*;
pub use property::*;
pub use class::*;
pub use godot_type::*;
pub use variant::*;
pub use variant_array::*;
pub use dictionary::*;
pub use geom::*;
pub use color::*;
pub use rid::*;
pub use node_path::*;
pub use generated::*;
pub use string::*;
pub use byte_array::*;
pub use string_array::*;
pub use vector2_array::*;
pub use vector3_array::*;
pub use color_array::*;

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


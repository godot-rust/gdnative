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
mod pool_byte_array;
mod pool_string_array;
mod pool_vector2_array;
mod pool_vector3_array;
mod pool_color_array;

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
pub use pool_byte_array::*;
pub use pool_string_array::*;
pub use pool_vector2_array::*;
pub use pool_vector3_array::*;
pub use pool_color_array::*;

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


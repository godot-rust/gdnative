
use super::*;
use std::sync::{Once, ONCE_INIT};
use std::ops::*;

pub use geom::*;

mod variant;
pub use self::variant::*;

pub unsafe trait GodotType: Sized {
    fn as_variant(&self) -> sys::godot_variant;
    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self>;
}

unsafe impl GodotType for () {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_nil)(&mut ret);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            if (get_api().godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_NIL {
                Some(())
            } else {
                None
            }
        }
    }
}

macro_rules! godot_int_impl {
    ($ty:ty) => (
        unsafe impl GodotType for $ty {
            fn as_variant(&self) -> sys::godot_variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_int)(&mut ret, *self as i64);
                    ret
                }
            }

            fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
                unsafe {
                    let api = get_api();
                    if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_INT {
                        Some((api.godot_variant_as_int)(variant) as Self)
                    } else {
                        None
                    }
                }
            }
        }
            )
}

godot_int_impl!(i8);
godot_int_impl!(i16);
godot_int_impl!(i32);
godot_int_impl!(i64);

macro_rules! godot_uint_impl {
    ($ty:ty) => (
        unsafe impl GodotType for $ty {
            fn as_variant(&self) -> sys::godot_variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_uint)(&mut ret, *self as u64);
                    ret
                }
            }

            fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
                unsafe {
                    let api = get_api();
                    if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_INT {
                        Some((api.godot_variant_as_uint)(variant) as Self)
                    } else {
                        None
                    }
                }
            }
        }
            )
}

godot_uint_impl!(u8);
godot_uint_impl!(u16);
godot_uint_impl!(u32);
godot_uint_impl!(u64);


unsafe impl GodotType for f32 {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, *self as f64);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_REAL {
                Some((api.godot_variant_as_real)(variant) as Self)
            } else {
                None
            }
        }
    }
}

unsafe impl GodotType for f64 {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, *self);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_REAL {
                Some((api.godot_variant_as_real)(variant) as Self)
            } else {
                None
            }
        }
    }
}

unsafe impl GodotType for String {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            let api = get_api();
            let mut string = (api.godot_string_chars_to_utf8_with_len)(self.as_ptr() as *const _, self.len() as _);
            (api.godot_variant_new_string)(&mut ret, &string);
            (api.godot_string_destroy)(&mut string);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_STRING {
                let mut variant = (api.godot_variant_as_string)(variant);
                let tmp = (api.godot_string_utf8)(&variant);
                let ret = ::std::ffi::CStr::from_ptr((api.godot_char_string_get_data)(&tmp) as *const _)
                    .to_string_lossy()
                    .into_owned();
                (api.godot_string_destroy)(&mut variant);
                Some(ret)
            } else {
                None
            }
        }
    }
}
#[derive(Clone, Copy)]
pub struct Color(sys::godot_color);

impl Color {
    pub fn new_rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        unsafe {
            let mut dest = sys::godot_color::default();
            (get_api().godot_color_new_rgba)(&mut dest, r, g, b, a);
            Color(dest)
        }
    }

    pub fn new_rgb(r: f32, g: f32, b: f32) -> Color {
        unsafe {
            let mut dest = sys::godot_color::default();
            (get_api().godot_color_new_rgb)(&mut dest, r, g, b);
            Color(dest)
        }
    }

    pub fn r(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_r)(&self.0)
        }
    }

    pub fn set_r(&mut self, v: f32) {
        unsafe {
            (get_api().godot_color_set_r)(&mut self.0, v)
        }
    }

    pub fn g(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_g)(&self.0)
        }
    }

    pub fn set_g(&mut self, v: f32) {
        unsafe {
            (get_api().godot_color_set_g)(&mut self.0, v)
        }
    }

    pub fn b(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_b)(&self.0)
        }
    }

    pub fn set_b(&mut self, v: f32) {
        unsafe {
            (get_api().godot_color_set_b)(&mut self.0, v)
        }
    }

    pub fn a(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_a)(&self.0)
        }
    }

    pub fn set_a(&mut self, v: f32) {
        unsafe {
            (get_api().godot_color_set_a)(&mut self.0, v)
        }
    }

    pub fn h(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_h)(&self.0)
        }
    }

    pub fn s(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_s)(&self.0)
        }
    }

    pub fn v(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_v)(&self.0)
        }
    }
}

pub struct NodePath(sys::godot_node_path);

impl NodePath {
    pub fn new(path: &str) -> NodePath {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            let api = get_api();
            let mut from = (api.godot_string_chars_to_utf8_with_len)(path.as_ptr() as *const _, path.len() as _);
            (api.godot_node_path_new)(&mut dest, &from);
            (api.godot_string_destroy)(&mut from);
            NodePath(dest)
        }
    }
}

impl Clone for NodePath {
    fn clone(&self) -> NodePath {
        unsafe {
            let mut dest = sys::godot_node_path::default();
            (get_api().godot_node_path_new_copy)(&mut dest, &self.0);
            NodePath(dest)
        }
    }
}

impl Drop for NodePath {
    fn drop(&mut self) {
        unsafe {
            (get_api().godot_node_path_destroy)(&mut self.0);
        }
    }
}

pub struct Nothing {
    info: GodotClassInfo,
}

unsafe impl GodotClass for Nothing {
    type Reference = Nothing;
    type ClassData = Nothing;

    fn godot_name() -> &'static str {
        ""
    }

    unsafe fn register_class(_desc: *mut libc::c_void) {
        panic!("Can't register");
    }

    fn godot_info(&self) -> &GodotClassInfo {
        &self.info
    }

    unsafe fn from_object(obj: *mut sys::godot_object) -> Self {
        Nothing {
            info: GodotClassInfo {
                this: obj,
            },
        }
    }
    unsafe fn reference(_this: *mut sys::godot_object, data: &Self::ClassData) -> &Self::Reference {
        data
    }
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));
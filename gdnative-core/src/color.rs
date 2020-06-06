use crate::private::get_api;
use crate::sys;
use std::mem::transmute;

/// RGBA color with 32 bits floating point components.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[inline]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[inline]
    pub fn h(&self) -> f32 {
        unsafe { (get_api().godot_color_get_h)(self.sys()) }
    }

    #[inline]
    pub fn s(&self) -> f32 {
        unsafe { (get_api().godot_color_get_s)(self.sys()) }
    }

    #[inline]
    pub fn v(&self) -> f32 {
        unsafe { (get_api().godot_color_get_v)(self.sys()) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> &sys::godot_color {
        unsafe { transmute(self) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn to_sys(self) -> sys::godot_color {
        unsafe { transmute(self) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_color) -> Self {
        unsafe { transmute::<sys::godot_color, Self>(c) }
    }
}

#[test]
fn color_repr() {
    use std::mem::size_of;
    assert_eq!(size_of::<Color>(), size_of::<sys::godot_color>());
}

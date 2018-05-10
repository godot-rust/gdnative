use sys;
use get_api;
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
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    fn as_sys_color(&self) -> &sys::godot_color {
        unsafe { transmute(self) }
    }

    pub fn h(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_h)(self.as_sys_color())
        }
    }

    pub fn s(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_s)(self.as_sys_color())
        }
    }

    pub fn v(&self) -> f32 {
        unsafe {
            (get_api().godot_color_get_v)(self.as_sys_color())
        }
    }
}

#[test]
fn color_repr() {
    use std::mem::size_of;
    assert_eq!(size_of::<Color>(), size_of::<sys::godot_color>());
}

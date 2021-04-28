use crate::private::get_api;
use crate::sys;
use std::mem::transmute;

use crate::core_types::GodotString;
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

    #[inline]
    pub fn lerp(&self, other: Color, weight: f32) -> Color {
        Color {
            r: self.r + (weight * (other.r - self.r)),
            g: self.g + (weight * (other.g - self.g)),
            b: self.b + (weight * (other.b - self.b)),
            a: self.a + (weight * (other.a - self.a)),
        }
    }
    #[inline]
    pub fn blend(&self, other: &Color) -> Color {
        Color::from_sys(unsafe { (get_api().godot_color_blend)(self.sys(), other.sys()) })
    }

    #[inline]
    pub fn contrasted(&self) -> Color {
        Color::from_sys(unsafe { (get_api().godot_color_contrasted)(self.sys()) })
    }
    #[inline]
    pub fn darkened(&self, amount: f32) -> Color {
        Color::from_sys(unsafe { (get_api().godot_color_darkened)(self.sys(), amount) })
    }
    #[inline]
    pub fn from_hsv(h: f32, s: f32, v: f32, a: f32) -> Color {
        let color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        Color::from_sys(unsafe { (get_api().godot_color_from_hsv)(color.sys(), h, s, v, a) })
    }
    #[inline]
    pub fn gray(&self) -> f32 {
        // Implemented as described in godot docs
        (self.r + self.b + self.g) / 3.0
    }

    #[inline]
    pub fn inverted(&self) -> Color {
        // Implementation as described in godot docs.
        Color {
            r: 1.0f32 - self.r,
            g: 1.0f32 - self.g,
            b: 1.0f32 - self.b,
            a: self.a,
        }
    }

    pub fn to_abgr32(&self) -> i32 {
        unsafe { (get_api().godot_color_to_abgr32)(self.sys()) }
    }

    pub fn to_abgr64(&self) -> i32 {
        unsafe { (get_api().godot_color_to_abgr64)(self.sys()) }
    }

    pub fn to_argb32(&self) -> i32 {
        unsafe { (get_api().godot_color_to_argb32)(self.sys()) }
    }

    pub fn to_argb64(&self) -> i32 {
        unsafe { (get_api().godot_color_to_argb64)(self.sys()) }
    }

    pub fn to_html(&self, with_alpha: bool) -> GodotString {
        GodotString::from_sys(unsafe { (get_api().godot_color_to_html)(self.sys(), with_alpha) })
    }

    pub fn to_rgba32(&self) -> i32 {
        unsafe { (get_api().godot_color_to_rgba32)(self.sys()) }
    }

    pub fn to_rgba64(&self) -> i32 {
        unsafe { (get_api().godot_color_to_rgba64)(self.sys()) }
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

godot_test!(test_color {
    // Test to_html
    assert_eq!("ffffffff", Color::rgba(1.0, 1.0, 1.0, 1.0).to_html(true).to_string());
    assert_eq!("ffffff", Color::rgba(1.0, 1.0, 1.0, 1.0).to_html(false).to_string());
    assert_eq!("80ffffff", Color::rgba(1.0, 1.0, 1.0, 0.5).to_html(true).to_string());
    assert_eq!("ffffff", Color::rgba(1.0, 1.0, 1.0, 0.5).to_html(false).to_string());
    assert_eq!("ff8000", Color::rgb(1.0, 0.5, 0.0).to_html(false).to_string());
    assert_eq!("ff0080ff", Color::rgb(0.0, 0.5, 1.0).to_html(true).to_string());
    // Test Gray
    // String comparison due to non-trivial way to truncate floats
    use crate::core_types::IsEqualApprox;
    assert!(0.4f32.is_equal_approx(Color::rgb(0.2, 0.4, 0.6).gray()));
    assert!(0.5f32.is_equal_approx(Color::rgb(0.1, 0.5, 0.9).gray()));
    assert!(0.9f32.is_equal_approx(Color::rgb(1.0, 1.0, 0.7).gray()));
    assert!(0.42f32.is_equal_approx(Color::rgb(0.6, 0.6, 0.06).gray()));
    // Test invert
    let inverted = Color::rgb(1.0, 1.0,1.0).inverted();
    assert!(0f32.is_equal_approx(inverted.r));
    assert!(0f32.is_equal_approx(inverted.g));
    assert!(0f32.is_equal_approx(inverted.b));

    let inverted = Color::rgb(0.95, 0.95,0.95).inverted();
    assert!(0.05f32.is_equal_approx(inverted.r));
    assert!(0.05f32.is_equal_approx(inverted.g));
    assert!(0.05f32.is_equal_approx(inverted.b));

    let inverted = Color::rgb(0.05, 0.95,0.55).inverted();
    assert!(0.95f32.is_equal_approx(inverted.r));
    assert!(0.05f32.is_equal_approx(inverted.g));
    assert!(0.45f32.is_equal_approx(inverted.b));

    // This is a series of sanity checks to test that the API bounds work properly.
    let color = Color::from_hsv(1.0, 1.0, 1.0, 1.0);
    color.darkened(0.20);
    color.contrasted();
    color.inverted();
    color.to_rgba32();
    color.to_rgba64();
    color.to_abgr32();
    color.to_abgr64();
    color.to_argb32();
    color.to_argb64();
    let other_color = Color::rgba(1.0, 1.0, 1.0, 1.0);
    color.blend(&other_color);
});

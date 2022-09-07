use crate::private::get_api;
use crate::sys;
use std::mem::transmute;

use crate::core_types::GodotString;

/// RGBA color with 32-bit floating point components.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[inline]
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[inline]
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[inline]
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Color {
        Color::from_hsva(h, s, v, 1.0)
    }

    #[inline]
    pub fn from_hsva(h: f32, s: f32, v: f32, a: f32) -> Color {
        let color = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
        Color::from_sys(unsafe { (get_api().godot_color_from_hsv)(color.sys(), h, s, v, a) })
    }

    /// Parses from a HTML color code, or `None` on parse error.
    ///
    /// Note that unlike most other constructors, this has `ARGB` and not `RGBA` format.
    /// In particular, `from_html("AB123456")` would correspond to `from_rgba_u32(0x123456AB)`.
    ///
    /// ```
    /// use gdnative::prelude::Color;
    ///
    /// let c1 = Color::from_html("#9eb2d90a"); // ARGB format with "#".
    /// let c2 = Color::from_html("9eb2d90a");  // ARGB format.
    /// let c3 = Color::from_html("#b2d90a");   // RGB format with "#".
    /// let c4 = Color::from_html("b2d90a");    // RGB format.
    ///
    /// let expected = Color::from_rgba_u8(0xb2, 0xd9, 0x0a, 0x9e);
    /// assert_eq!(c1, Some(expected));
    /// assert_eq!(c2, Some(expected));
    ///
    /// let expected = Color::from_rgba_u8(0xb2, 0xd9, 0x0a, 0xff);
    /// assert_eq!(c3, Some(expected));
    /// assert_eq!(c4, Some(expected));
    /// ```
    ///
    /// See also the corresponding [GDScript method](https://docs.godotengine.org/en/stable/classes/class_color.html#class-color-method-color).
    #[inline]
    pub fn from_html(mut argb_or_rgb: &str) -> Option<Self> {
        if let Some(stripped) = argb_or_rgb.strip_prefix('#') {
            argb_or_rgb = stripped;
        }

        let (rgb_str, a) = match argb_or_rgb.len() {
            6 => (argb_or_rgb, 0xFF),
            8 => {
                let (a_str, rgb_str) = argb_or_rgb.split_at(2);
                if let Ok(a) = u8::from_str_radix(a_str, 16) {
                    (rgb_str, a)
                } else {
                    return None;
                }
            }
            _ => return None,
        };

        if let Ok(rgb) = u32::from_str_radix(rgb_str, 16) {
            let color = Self::from_rgba_u32(rgb << 8 | a as u32);
            Some(color)
        } else {
            None
        }
    }

    /// Construct a color from a single `u32` value, in `RGBA` format.
    ///
    /// Example:
    /// ```
    /// use gdnative::prelude::Color;
    ///
    /// // RGBA (178, 217, 10, 158)
    /// let one = Color::from_rgba_u32(0xb2d90a9e);
    /// let piecewise = Color::from_rgba_u8(0xB2, 0xD9, 0x0A, 0x9E);
    /// assert_eq!(one, piecewise);
    /// ```
    #[inline]
    pub fn from_rgba_u32(rgba: u32) -> Self {
        Self::from_rgba_u8(
            (rgba >> 24) as u8,
            ((rgba >> 16) & 0xFF) as u8,
            ((rgba >> 8) & 0xFF) as u8,
            (rgba & 0xFF) as u8,
        )
    }

    /// Constructs a color from four integer channels, each in range 0-255.
    ///
    /// This corresponds to the
    /// [GDScript method `Color8`](https://docs.godotengine.org/en/stable/classes/class_%40gdscript.html#class-gdscript-method-color8).
    #[inline]
    pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from_rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
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
    pub fn gray(&self) -> f32 {
        // Implemented as described in godot docs
        (self.r + self.b + self.g) / 3.0
    }

    #[inline]
    pub fn inverted(&self) -> Color {
        // Implementation as described in godot docs.
        Color {
            r: 1.0 - self.r,
            g: 1.0 - self.g,
            b: 1.0 - self.b,
            a: self.a,
        }
    }

    #[inline]
    pub fn to_html(self, with_alpha: bool) -> GodotString {
        GodotString::from_sys(unsafe { (get_api().godot_color_to_html)(self.sys(), with_alpha) })
    }

    /// Returns the reverse of the RGBA32 byte representation for this color where each byte represents a component of the ABGR profile.
    /// This is the byte information used when storing this color as a part of a texture.
    /// # Endianness
    /// On big endian architecture this is stored in ABGR byte order
    /// On little endian machines this is stored in RGBA byte order
    /// # Example
    /// `0x00FF7FFF` would be the equivalent to `Color::from_rgba(1.0, 0.5, 1.0, 0.0)`
    #[inline]
    pub fn to_abgr32(self) -> u32 {
        ((self.a * 255.0) as u32) << 24
            | ((self.b * 255.0) as u32) << 16
            | ((self.g * 255.0) as u32) << 8
            | (self.r * 255.0) as u32
    }

    /// Returns the reverse of the RGBA64 byte representation for this color where each word represents represents a component of the ABGR profile.
    /// This is the byte information used when storing this color as a part of a texture.
    /// # Endianness
    /// On big endian architecture this is stored in ABGR word order
    /// On little endian machines this is stored in RGBA word order
    /// # Example
    /// `0x0000FFFF7FFFFFFF` would be the equivalent to `Color::from_rgba(0.0, 1.0, 0.5, 1.0)`
    #[inline]
    pub fn to_abgr64(self) -> u64 {
        ((self.a * 65535.0) as u64) << 48
            | ((self.b * 65535.0) as u64) << 32
            | ((self.g * 65535.0) as u64) << 16
            | ((self.r * 65535.0) as u64)
    }

    /// Returns the ARGB32 format representation representation for this color where each byte represents a component of the ARGB profile.
    /// This is the byte information used when storing this color as a part of a texture.
    /// # Endianness
    /// On big endian architecture this is stored in the order ARGB byte order
    /// On little endian machines this is stored in the order BGRA byte order
    /// `0x0000FFFF7FFFFFFF` would be the equivalent to `Color::from_rgba(1.0, 0.5, 1.0, 0.0)`
    #[inline]
    pub fn to_argb32(self) -> u32 {
        ((self.a * 255.0) as u32) << 24
            | ((self.r * 255.0) as u32) << 16
            | ((self.g * 255.0) as u32) << 8
            | (self.b * 255.0) as u32
    }

    /// Returns the ARGB64 format representation for this color where each word represents a component of the ARGB profile.
    /// This is the byte information used when storing this color as a part of a texture.
    /// # Endianness
    /// On big endian architecture this is stored in the order ARGB word order
    /// On little endian machines this is stored in the order BGRA word order
    /// # Example
    /// `0x0000FFFF7FFFFFFF` would be the equivalent to `Color::from_rgba(1.0, 0.5, 1.0, 0.0)`
    #[inline]
    pub fn to_argb64(self) -> u64 {
        ((self.a * 65535.0) as u64) << 48
            | ((self.r * 65535.0) as u64) << 32
            | ((self.g * 65535.0) as u64) << 16
            | ((self.b * 65535.0) as u64)
    }

    /// Returns the OpenGL Texture format byte representation for this color where each byte represents a component of the RGBA profile.
    /// This is the byte information used when storing this color as a part of a texture.
    /// # Endianness
    /// On big endian architecture this is stored in RGBA byte order
    /// On little endian machines this is stored in ABGR byte order
    /// # Example
    /// `0x00FF7FFF` would be the equivalent to `Color::from_rgba(0.0, 1.0, 0.5, 1.0)`
    #[inline]
    pub fn to_rgba32(self) -> u32 {
        ((self.r * 255.0) as u32) << 24
            | ((self.g * 255.0) as u32) << 16
            | ((self.b * 255.0) as u32) << 8
            | (self.a * 255.0) as u32
    }

    /// Returns the OpenGL Texture format byte representation for this color where each byte represents a component of the RGBA profile.
    /// This is the byte information used when storing this color as a part of a texture.
    /// # Endianness
    /// On big endian architecture this is stored in RGBA word order
    /// On little endian machines this is stored in ABGR word order
    /// # Example
    /// `0x0000FFFF7FFFFFFF` would be the equivalent to `Color::from_rgba(0.0, 1.0, 0.5, 1.0)`
    #[inline]
    pub fn to_rgba64(self) -> u64 {
        ((self.r * 65535.0) as u64) << 48
            | ((self.g * 65535.0) as u64) << 32
            | ((self.b * 65535.0) as u64) << 16
            | ((self.a * 65535.0) as u64)
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

#[test]
fn color_to_pixel_color_formats() {
    let color = Color::from_rgba(1.0, 0.5, 1.0, 0.0);
    assert_eq!(0xFF7FFF00, color.to_rgba32());
    assert_eq!(0xFFFF7FFFFFFF0000, color.to_rgba64());
    assert_eq!(0x00FF7FFF, color.to_abgr32());
    assert_eq!(0x0000FFFF7FFFFFFF, color.to_abgr64());
    assert_eq!(0x00FF7FFF, color.to_argb32());
    assert_eq!(0x0000FFFF7FFFFFFF, color.to_argb64());
}

godot_test!(test_color {
    // Test to_html
    assert_eq!("ffffffff", Color::from_rgba(1.0, 1.0, 1.0, 1.0).to_html(true).to_string());
    assert_eq!("ffffff", Color::from_rgba(1.0, 1.0, 1.0, 1.0).to_html(false).to_string());
    assert_eq!("80ffffff", Color::from_rgba(1.0, 1.0, 1.0, 0.5).to_html(true).to_string());
    assert_eq!("ffffff", Color::from_rgba(1.0, 1.0, 1.0, 0.5).to_html(false).to_string());
    assert_eq!("ff8000", Color::from_rgb(1.0, 0.5, 0.0).to_html(false).to_string());
    assert_eq!("ff0080ff", Color::from_rgb(0.0, 0.5, 1.0).to_html(true).to_string());

    // Test Gray
    // String comparison due to non-trivial way to truncate floats
    use crate::core_types::IsEqualApprox;
    assert!(0.4f32.is_equal_approx(Color::from_rgb(0.2, 0.4, 0.6).gray()));
    assert!(0.5f32.is_equal_approx(Color::from_rgb(0.1, 0.5, 0.9).gray()));
    assert!(0.9f32.is_equal_approx(Color::from_rgb(1.0, 1.0, 0.7).gray()));
    assert!(0.42f32.is_equal_approx(Color::from_rgb(0.6, 0.6, 0.06).gray()));
    // Test invert
    let inverted = Color::from_rgb(1.0, 1.0,1.0).inverted();
    assert!(0f32.is_equal_approx(inverted.r));
    assert!(0f32.is_equal_approx(inverted.g));
    assert!(0f32.is_equal_approx(inverted.b));

    let inverted = Color::from_rgb(0.95, 0.95,0.95).inverted();
    assert!(0.05f32.is_equal_approx(inverted.r));
    assert!(0.05f32.is_equal_approx(inverted.g));
    assert!(0.05f32.is_equal_approx(inverted.b));

    let inverted = Color::from_rgb(0.05, 0.95,0.55).inverted();
    assert!(0.95f32.is_equal_approx(inverted.r));
    assert!(0.05f32.is_equal_approx(inverted.g));
    assert!(0.45f32.is_equal_approx(inverted.b));

    // This is a series of sanity checks to test that the API bounds work properly.
    let hsv_color = Color::from_hsv(0.75, 0.5, 0.25);
    let color = Color::from_hsva(0.75, 0.5, 0.25, 1.0);
    assert_eq!(hsv_color, color);
    let color = Color::from_rgb(0.75, 0.5, 0.25);
    assert_eq!(Color::from_rgb(0.25, 0.5, 0.75), color.inverted());
    // Following results were derived from the godot engine code based on the RGB values of 0.75, 0.5, 0.25 respectively.
    assert_eq!(Color::from_rgb(0.25, 0.00, 0.75), color.contrasted());
    assert_eq!(Color::from_rgba(0.60, 0.40, 0.20, 1.0), color.darkened(0.20));
    // Check that the blend values are correct.
    let color = Color::from_rgba(0.0, 1.0, 0.5, 1.0);
    let other_color = Color::from_rgba(1.0, 0.0, 0.5, 1.0);
    assert_eq!(Color::from_rgba(1.0, 0.0, 0.5, 1.0), color.blend(&other_color));
});

use crate::private::get_api;
use crate::sys;
use std::mem::transmute;

use crate::core_types::GodotString;

// Color Constants porting from https://docs.godotengine.org/en/stable/classes/class_color.html#constants
pub const ALICEBLUE: Color = Color {
    r: 0.94,
    g: 0.97,
    b: 1.0,
    a: 1.0,
};
pub const ANTIQUEWHITE: Color = Color {
    r: 0.98,
    g: 0.92,
    b: 0.84,
    a: 1.0,
};
pub const AQUA: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const AQUAMARINE: Color = Color {
    r: 0.5,
    g: 1.0,
    b: 0.83,
    a: 1.0,
};
pub const AZURE: Color = Color {
    r: 0.94,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const BEIGE: Color = Color {
    r: 0.96,
    g: 0.96,
    b: 0.86,
    a: 1.0,
};
pub const BISQUE: Color = Color {
    r: 1.0,
    g: 0.89,
    b: 0.77,
    a: 1.0,
};
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const BLANCHEDALMOND: Color = Color {
    r: 1.0,
    g: 0.92,
    b: 0.8,
    a: 1.0,
};
pub const BLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
pub const BLUEVIOLET: Color = Color {
    r: 0.54,
    g: 0.17,
    b: 0.89,
    a: 1.0,
};
pub const BROWN: Color = Color {
    r: 0.65,
    g: 0.16,
    b: 0.16,
    a: 1.0,
};
pub const BURLYWOOD: Color = Color {
    r: 0.87,
    g: 0.72,
    b: 0.53,
    a: 1.0,
};
pub const CADETBLUE: Color = Color {
    r: 0.37,
    g: 0.62,
    b: 0.63,
    a: 1.0,
};
pub const CHARTREUSE: Color = Color {
    r: 0.5,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const CHOCOLATE: Color = Color {
    r: 0.82,
    g: 0.41,
    b: 0.12,
    a: 1.0,
};
pub const CORAL: Color = Color {
    r: 1.0,
    g: 0.5,
    b: 0.31,
    a: 1.0,
};
pub const CORNFLOWER: Color = Color {
    r: 0.39,
    g: 0.58,
    b: 0.93,
    a: 1.0,
};
pub const CORNSILK: Color = Color {
    r: 1.0,
    g: 0.97,
    b: 0.86,
    a: 1.0,
};
pub const CRIMSON: Color = Color {
    r: 0.86,
    g: 0.08,
    b: 0.24,
    a: 1.0,
};
pub const CYAN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const DARKBLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.55,
    a: 1.0,
};
pub const DARKCYAN: Color = Color {
    r: 0.0,
    g: 0.55,
    b: 0.55,
    a: 1.0,
};
pub const DARKGOLDENROD: Color = Color {
    r: 0.72,
    g: 0.53,
    b: 0.04,
    a: 1.0,
};
pub const DARKGRAY: Color = Color {
    r: 0.66,
    g: 0.66,
    b: 0.66,
    a: 1.0,
};
pub const DARKGREEN: Color = Color {
    r: 0.0,
    g: 0.39,
    b: 0.0,
    a: 1.0,
};
pub const DARKKHAKI: Color = Color {
    r: 0.74,
    g: 0.72,
    b: 0.42,
    a: 1.0,
};
pub const DARKMAGENTA: Color = Color {
    r: 0.55,
    g: 0.0,
    b: 0.55,
    a: 1.0,
};
pub const DARKOLIVEGREEN: Color = Color {
    r: 0.33,
    g: 0.42,
    b: 0.18,
    a: 1.0,
};
pub const DARKORANGE: Color = Color {
    r: 1.0,
    g: 0.55,
    b: 0.0,
    a: 1.0,
};
pub const DARKORCHID: Color = Color {
    r: 0.6,
    g: 0.2,
    b: 0.8,
    a: 1.0,
};
pub const DARKRED: Color = Color {
    r: 0.55,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const DARKSALMON: Color = Color {
    r: 0.91,
    g: 0.59,
    b: 0.48,
    a: 1.0,
};
pub const DARKSEAGREEN: Color = Color {
    r: 0.56,
    g: 0.74,
    b: 0.56,
    a: 1.0,
};
pub const DARKSLATEBLUE: Color = Color {
    r: 0.28,
    g: 0.24,
    b: 0.55,
    a: 1.0,
};
pub const DARKSLATEGRAY: Color = Color {
    r: 0.18,
    g: 0.31,
    b: 0.31,
    a: 1.0,
};
pub const DARKTURQUOISE: Color = Color {
    r: 0.0,
    g: 0.81,
    b: 0.82,
    a: 1.0,
};
pub const DARKVIOLET: Color = Color {
    r: 0.58,
    g: 0.0,
    b: 0.83,
    a: 1.0,
};
pub const DEEPPINK: Color = Color {
    r: 1.0,
    g: 0.08,
    b: 0.58,
    a: 1.0,
};
pub const DEEPSKYBLUE: Color = Color {
    r: 0.0,
    g: 0.75,
    b: 1.0,
    a: 1.0,
};
pub const DIMGRAY: Color = Color {
    r: 0.41,
    g: 0.41,
    b: 0.41,
    a: 1.0,
};
pub const DODGERBLUE: Color = Color {
    r: 0.12,
    g: 0.56,
    b: 1.0,
    a: 1.0,
};
pub const FIREBRICK: Color = Color {
    r: 0.7,
    g: 0.13,
    b: 0.13,
    a: 1.0,
};
pub const FLORALWHITE: Color = Color {
    r: 1.0,
    g: 0.98,
    b: 0.94,
    a: 1.0,
};
pub const FORESTGREEN: Color = Color {
    r: 0.13,
    g: 0.55,
    b: 0.13,
    a: 1.0,
};
pub const FUCHSIA: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
pub const GAINSBORO: Color = Color {
    r: 0.86,
    g: 0.86,
    b: 0.86,
    a: 1.0,
};
pub const GHOSTWHITE: Color = Color {
    r: 0.97,
    g: 0.97,
    b: 1.0,
    a: 1.0,
};
pub const GOLD: Color = Color {
    r: 1.0,
    g: 0.84,
    b: 0.0,
    a: 1.0,
};
pub const GOLDENROD: Color = Color {
    r: 0.85,
    g: 0.65,
    b: 0.13,
    a: 1.0,
};
pub const GRAY: Color = Color {
    r: 0.75,
    g: 0.75,
    b: 0.75,
    a: 1.0,
};
pub const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const GREENYELLOW: Color = Color {
    r: 0.68,
    g: 1.0,
    b: 0.18,
    a: 1.0,
};
pub const HONEYDEW: Color = Color {
    r: 0.94,
    g: 1.0,
    b: 0.94,
    a: 1.0,
};
pub const HOTPINK: Color = Color {
    r: 1.0,
    g: 0.41,
    b: 0.71,
    a: 1.0,
};
pub const INDIANRED: Color = Color {
    r: 0.8,
    g: 0.36,
    b: 0.36,
    a: 1.0,
};
pub const INDIGO: Color = Color {
    r: 0.29,
    g: 0.0,
    b: 0.51,
    a: 1.0,
};
pub const IVORY: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.94,
    a: 1.0,
};
pub const KHAKI: Color = Color {
    r: 0.94,
    g: 0.9,
    b: 0.55,
    a: 1.0,
};
pub const LAVENDER: Color = Color {
    r: 0.9,
    g: 0.9,
    b: 0.98,
    a: 1.0,
};
pub const LAVENDERBLUSH: Color = Color {
    r: 1.0,
    g: 0.94,
    b: 0.96,
    a: 1.0,
};
pub const LAWNGREEN: Color = Color {
    r: 0.49,
    g: 0.99,
    b: 0.0,
    a: 1.0,
};
pub const LEMONCHIFFON: Color = Color {
    r: 1.0,
    g: 0.98,
    b: 0.8,
    a: 1.0,
};
pub const LIGHTBLUE: Color = Color {
    r: 0.68,
    g: 0.85,
    b: 0.9,
    a: 1.0,
};
pub const LIGHTCORAL: Color = Color {
    r: 0.94,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
pub const LIGHTCYAN: Color = Color {
    r: 0.88,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const LIGHTGOLDENROD: Color = Color {
    r: 0.98,
    g: 0.98,
    b: 0.82,
    a: 1.0,
};
pub const LIGHTGRAY: Color = Color {
    r: 0.83,
    g: 0.83,
    b: 0.83,
    a: 1.0,
};
pub const LIGHTGREEN: Color = Color {
    r: 0.56,
    g: 0.93,
    b: 0.56,
    a: 1.0,
};
pub const LIGHTPINK: Color = Color {
    r: 1.0,
    g: 0.71,
    b: 0.76,
    a: 1.0,
};
pub const LIGHTSALMON: Color = Color {
    r: 1.0,
    g: 0.63,
    b: 0.48,
    a: 1.0,
};
pub const LIGHTSEAGREEN: Color = Color {
    r: 0.13,
    g: 0.7,
    b: 0.67,
    a: 1.0,
};
pub const LIGHTSKYBLUE: Color = Color {
    r: 0.53,
    g: 0.81,
    b: 0.98,
    a: 1.0,
};
pub const LIGHTSLATEGRAY: Color = Color {
    r: 0.47,
    g: 0.53,
    b: 0.6,
    a: 1.0,
};
pub const LIGHTSTEELBLUE: Color = Color {
    r: 0.69,
    g: 0.77,
    b: 0.87,
    a: 1.0,
};
pub const LIGHTYELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.88,
    a: 1.0,
};
pub const LIME: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const LIMEGREEN: Color = Color {
    r: 0.2,
    g: 0.8,
    b: 0.2,
    a: 1.0,
};
pub const LINEN: Color = Color {
    r: 0.98,
    g: 0.94,
    b: 0.9,
    a: 1.0,
};
pub const MAGENTA: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
pub const MAROON: Color = Color {
    r: 0.69,
    g: 0.19,
    b: 0.38,
    a: 1.0,
};
pub const MEDIUMAQUAMARINE: Color = Color {
    r: 0.4,
    g: 0.8,
    b: 0.67,
    a: 1.0,
};
pub const MEDIUMBLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.8,
    a: 1.0,
};
pub const MEDIUMORCHID: Color = Color {
    r: 0.73,
    g: 0.33,
    b: 0.83,
    a: 1.0,
};
pub const MEDIUMPURPLE: Color = Color {
    r: 0.58,
    g: 0.44,
    b: 0.86,
    a: 1.0,
};
pub const MEDIUMSEAGREEN: Color = Color {
    r: 0.24,
    g: 0.7,
    b: 0.44,
    a: 1.0,
};
pub const MEDIUMSLATEBLUE: Color = Color {
    r: 0.48,
    g: 0.41,
    b: 0.93,
    a: 1.0,
};
pub const MEDIUMSPRINGGREEN: Color = Color {
    r: 0.0,
    g: 0.98,
    b: 0.6,
    a: 1.0,
};
pub const MEDIUMTURQUOISE: Color = Color {
    r: 0.28,
    g: 0.82,
    b: 0.8,
    a: 1.0,
};
pub const MEDIUMVIOLETRED: Color = Color {
    r: 0.78,
    g: 0.08,
    b: 0.52,
    a: 1.0,
};
pub const MIDNIGHTBLUE: Color = Color {
    r: 0.1,
    g: 0.1,
    b: 0.44,
    a: 1.0,
};
pub const MINTCREAM: Color = Color {
    r: 0.96,
    g: 1.0,
    b: 0.98,
    a: 1.0,
};
pub const MISTYROSE: Color = Color {
    r: 1.0,
    g: 0.89,
    b: 0.88,
    a: 1.0,
};
pub const MOCCASIN: Color = Color {
    r: 1.0,
    g: 0.89,
    b: 0.71,
    a: 1.0,
};
pub const NAVAJOWHITE: Color = Color {
    r: 1.0,
    g: 0.87,
    b: 0.68,
    a: 1.0,
};
pub const NAVYBLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.5,
    a: 1.0,
};
pub const OLDLACE: Color = Color {
    r: 0.99,
    g: 0.96,
    b: 0.9,
    a: 1.0,
};
pub const OLIVE: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.0,
    a: 1.0,
};
pub const OLIVEDRAB: Color = Color {
    r: 0.42,
    g: 0.56,
    b: 0.14,
    a: 1.0,
};
pub const ORANGE: Color = Color {
    r: 1.0,
    g: 0.65,
    b: 0.0,
    a: 1.0,
};
pub const ORANGERED: Color = Color {
    r: 1.0,
    g: 0.27,
    b: 0.0,
    a: 1.0,
};
pub const ORCHID: Color = Color {
    r: 0.85,
    g: 0.44,
    b: 0.84,
    a: 1.0,
};
pub const PALEGOLDENROD: Color = Color {
    r: 0.93,
    g: 0.91,
    b: 0.67,
    a: 1.0,
};
pub const PALEGREEN: Color = Color {
    r: 0.6,
    g: 0.98,
    b: 0.6,
    a: 1.0,
};
pub const PALETURQUOISE: Color = Color {
    r: 0.69,
    g: 0.93,
    b: 0.93,
    a: 1.0,
};
pub const PALEVIOLETRED: Color = Color {
    r: 0.86,
    g: 0.44,
    b: 0.58,
    a: 1.0,
};
pub const PAPAYAWHIP: Color = Color {
    r: 1.0,
    g: 0.94,
    b: 0.84,
    a: 1.0,
};
pub const PEACHPUFF: Color = Color {
    r: 1.0,
    g: 0.85,
    b: 0.73,
    a: 1.0,
};
pub const PERU: Color = Color {
    r: 0.8,
    g: 0.52,
    b: 0.25,
    a: 1.0,
};
pub const PINK: Color = Color {
    r: 1.0,
    g: 0.75,
    b: 0.8,
    a: 1.0,
};
pub const PLUM: Color = Color {
    r: 0.87,
    g: 0.63,
    b: 0.87,
    a: 1.0,
};
pub const POWDERBLUE: Color = Color {
    r: 0.69,
    g: 0.88,
    b: 0.9,
    a: 1.0,
};
pub const PURPLE: Color = Color {
    r: 0.63,
    g: 0.13,
    b: 0.94,
    a: 1.0,
};
pub const REBECCAPURPLE: Color = Color {
    r: 0.4,
    g: 0.2,
    b: 0.6,
    a: 1.0,
};
pub const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const ROSYBROWN: Color = Color {
    r: 0.74,
    g: 0.56,
    b: 0.56,
    a: 1.0,
};
pub const ROYALBLUE: Color = Color {
    r: 0.25,
    g: 0.41,
    b: 0.88,
    a: 1.0,
};
pub const SADDLEBROWN: Color = Color {
    r: 0.55,
    g: 0.27,
    b: 0.07,
    a: 1.0,
};
pub const SALMON: Color = Color {
    r: 0.98,
    g: 0.5,
    b: 0.45,
    a: 1.0,
};
pub const SANDYBROWN: Color = Color {
    r: 0.96,
    g: 0.64,
    b: 0.38,
    a: 1.0,
};
pub const SEAGREEN: Color = Color {
    r: 0.18,
    g: 0.55,
    b: 0.34,
    a: 1.0,
};
pub const SEASHELL: Color = Color {
    r: 1.0,
    g: 0.96,
    b: 0.93,
    a: 1.0,
};
pub const SIENNA: Color = Color {
    r: 0.63,
    g: 0.32,
    b: 0.18,
    a: 1.0,
};
pub const SILVER: Color = Color {
    r: 0.75,
    g: 0.75,
    b: 0.75,
    a: 1.0,
};
pub const SKYBLUE: Color = Color {
    r: 0.53,
    g: 0.81,
    b: 0.92,
    a: 1.0,
};
pub const SLATEBLUE: Color = Color {
    r: 0.42,
    g: 0.35,
    b: 0.8,
    a: 1.0,
};
pub const SLATEGRAY: Color = Color {
    r: 0.44,
    g: 0.5,
    b: 0.56,
    a: 1.0,
};
pub const SNOW: Color = Color {
    r: 1.0,
    g: 0.98,
    b: 0.98,
    a: 1.0,
};
pub const SPRINGGREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.5,
    a: 1.0,
};
pub const STEELBLUE: Color = Color {
    r: 0.27,
    g: 0.51,
    b: 0.71,
    a: 1.0,
};
pub const TAN: Color = Color {
    r: 0.82,
    g: 0.71,
    b: 0.55,
    a: 1.0,
};
pub const TEAL: Color = Color {
    r: 0.0,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
pub const THISTLE: Color = Color {
    r: 0.85,
    g: 0.75,
    b: 0.85,
    a: 1.0,
};
pub const TOMATO: Color = Color {
    r: 1.0,
    g: 0.39,
    b: 0.28,
    a: 1.0,
};
pub const TRANSPARENT: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.0,
};
pub const TURQUOISE: Color = Color {
    r: 0.25,
    g: 0.88,
    b: 0.82,
    a: 1.0,
};
pub const VIOLET: Color = Color {
    r: 0.93,
    g: 0.51,
    b: 0.93,
    a: 1.0,
};
pub const WEBGRAY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
pub const WEBGREEN: Color = Color {
    r: 0.0,
    g: 0.5,
    b: 0.0,
    a: 1.0,
};
pub const WEBMAROON: Color = Color {
    r: 0.5,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
pub const WEBPURPLE: Color = Color {
    r: 0.5,
    g: 0.0,
    b: 0.5,
    a: 1.0,
};
pub const WHEAT: Color = Color {
    r: 0.96,
    g: 0.87,
    b: 0.7,
    a: 1.0,
};
pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const WHITESMOKE: Color = Color {
    r: 0.96,
    g: 0.96,
    b: 0.96,
    a: 1.0,
};
pub const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
pub const YELLOWGREEN: Color = Color {
    r: 0.6,
    g: 0.8,
    b: 0.2,
    a: 1.0,
};

// RGBA color with 32 bits floating point components.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[deprecated]
    #[inline]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[deprecated]
    #[inline]
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

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
    pub fn to_html(&self, with_alpha: bool) -> GodotString {
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
    pub fn to_abgr32(&self) -> u32 {
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
    pub fn to_abgr64(&self) -> u64 {
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
    pub fn to_argb32(&self) -> u32 {
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
    pub fn to_argb64(&self) -> u64 {
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
    pub fn to_rgba32(&self) -> u32 {
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
    pub fn to_rgba64(&self) -> u64 {
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

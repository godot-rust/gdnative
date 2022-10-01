use super::IsEqualApprox;
use glam::Vec2;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 2D vector class.
///
/// See also [Vector2](https://docs.godotengine.org/en/stable/classes/class_vector2.html) in the Godot API doc.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

/// Helper methods for `Vector2`.
///
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/3.1/classes/class_vector2.html).
impl Vector2 {
    /// The zero vector.
    pub const ZERO: Vector2 = Vector2::new(0.0, 0.0);

    /// A vector with all components set to 1. Typically used for scaling.
    pub const ONE: Vector2 = Vector2::new(1.0, 1.0);

    /// A vector with all components set to +infinity.
    pub const INF: Vector2 = Vector2::new(f32::INFINITY, f32::INFINITY);

    /// Unit vector in -X direction.
    pub const LEFT: Vector2 = Vector2::new(-1.0, 0.0);

    /// Unit vector in +X direction.
    pub const RIGHT: Vector2 = Vector2::new(1.0, 0.0);

    /// Unit vector in -Y direction (the Y axis points down in 2D).
    pub const UP: Vector2 = Vector2::new(0.0, -1.0);

    /// Unit vector in +Y direction (the Y axis points down in 2D).
    pub const DOWN: Vector2 = Vector2::new(0.0, 1.0);

    /// Constructs a new Vector2 from the given x and y.
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns a new vector with all components in absolute values (i.e. positive).
    #[inline]
    pub fn abs(self) -> Self {
        Self::gd(self.glam().abs())
    }

    /// Returns this vector's angle with respect to the positive X axis, or (1, 0) vector, in
    /// radians.
    ///
    /// For example, Vector2.RIGHT.angle() will return zero, Vector2.DOWN.angle() will return PI / 2
    /// (a quarter turn, or 90 degrees), and Vector2(1, -1).angle() will return -PI / 4 (a negative
    /// eighth turn, or -45 degrees).
    ///
    /// Equivalent to the result of @GDScript.atan2 when called with the vector's y and x as
    /// parameters: atan2(y, x).
    #[inline]
    pub fn angle(self) -> f32 {
        self.glam().angle_between(Vec2::X)
    }

    /// Returns the angle to the given vector, in radians.
    #[inline]
    pub fn angle_to(self, to: Self) -> f32 {
        self.glam().angle_between(to.glam())
    }

    /// Returns the angle between the line connecting the two points and the X axis, in radians
    #[inline]
    pub fn angle_to_point(self, to: Self) -> f32 {
        self.glam().angle_between(to.glam() - self.glam())
    }

    /// Returns the aspect ratio of this vector, the ratio of x to y.
    #[inline]
    pub fn aspect(self) -> f32 {
        self.x / self.y
    }

    /// Returns the vector "bounced off" from a plane defined by the given normal.
    #[inline]
    pub fn bounce(self, n: Self) -> Self {
        -self.reflect(n)
    }

    /// Returns the vector with all components rounded up (towards positive infinity).
    #[inline]
    pub fn ceil(self) -> Self {
        Self::gd(self.glam().ceil())
    }

    /// Returns the vector with a maximum length by limiting its length to `length`.
    #[inline]
    pub fn clamped(self, length: f32) -> Self {
        Self::gd(self.glam().clamp_length_max(length))
    }

    /// Returns the cross product of this vector and `with`.
    #[inline]
    pub fn cross(self, with: Self) -> f32 {
        self.x * with.y - self.y * with.x
    }

    /// Cubicly interpolates between this vector and `b` using `pre_a` and `post_b` as handles,
    /// and returns the result at position `t`. `t` is in the range of 0.0 - 1.0, representing
    /// the amount of interpolation.
    #[inline]
    pub fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self {
        let v0 = pre_a;
        let v1 = self;
        let v2 = b;
        let v3 = post_b;

        let t2 = t * t;
        let t3 = t2 * t;

        ((v1 * 2.0)
            + (-v0 + v2) * t
            + (v0 * 2.0 - v1 * 5.0 + v2 * 4.0 - v3) * t2
            + (-v0 + v1 * 3.0 - v2 * 3.0 + v3) * t3)
            * 0.5
    }

    /// Returns the normalized vector pointing from this vector to `other`.
    #[inline]
    pub fn direction_to(self, other: Self) -> Self {
        (other - self).normalized()
    }

    /// Returns the squared distance to `other`.
    ///
    /// This method runs faster than distance_to, so prefer it if you need to compare vectors or
    /// need the squared distance for some formula.
    #[inline]
    pub fn distance_squared_to(self, other: Self) -> f32 {
        self.glam().distance_squared(other.glam())
    }

    /// Returns the distance to `other`.
    #[inline]
    pub fn distance_to(self, other: Self) -> f32 {
        self.glam().distance(other.glam())
    }

    /// Returns the dot product of this vector and `with`. This can be used to compare the angle
    /// between two vectors. For example, this can be used to determine whether an enemy is facing
    /// the player.
    ///
    /// The dot product will be 0 for a straight angle (90 degrees), greater than 0 for angles
    /// narrower than 90 degrees and lower than 0 for angles wider than 90 degrees.
    ///
    /// When using unit (normalized) vectors, the result will always be between -1.0 (180 degree
    /// angle) when the vectors are facing opposite directions, and 1.0 (0 degree angle) when the
    /// vectors are aligned.
    ///
    /// Note: `a.dot(b)` is equivalent to `b.dot(a)`.
    #[inline]
    pub fn dot(self, with: Self) -> f32 {
        self.glam().dot(with.glam())
    }

    /// Returns the vector with all components rounded down (towards negative infinity).
    #[inline]
    pub fn floor(self) -> Self {
        Self::gd(self.glam().floor())
    }

    /// Returns true if this vector and v are approximately equal, by running
    /// `@GDScript.is_equal_approx` on each component.
    #[inline]
    pub fn is_equal_approx(self, v: Self) -> bool {
        self.x.is_equal_approx(v.x) && self.y.is_equal_approx(v.y)
    }

    /// Returns `true` if the vector is normalized, and false otherwise.
    #[inline]
    pub fn is_normalized(self) -> bool {
        self.glam().is_normalized()
    }

    /// Returns the length (magnitude) of this vector.
    #[inline]
    pub fn length(self) -> f32 {
        self.glam().length()
    }

    /// Returns the squared length (squared magnitude) of this vector.
    ///
    /// This method runs faster than length, so prefer it if you need to compare vectors or need
    /// the squared distance for some formula.
    #[inline]
    pub fn length_squared(self) -> f32 {
        self.glam().length_squared()
    }

    /// Returns the result of the linear interpolation between this vector and b by amount t. t is
    /// on the range of 0.0 to 1.0, representing the amount of interpolation.
    #[inline]
    pub fn linear_interpolate(self, b: Self, t: f32) -> Self {
        Self::gd(self.glam().lerp(b.glam(), t))
    }

    /// Returns `self` moved towards `to` by the distance `delta`, clamped by `to`.
    #[inline]
    pub fn move_toward(self, to: Vector2, delta: f32) -> Self {
        let vd = to - self;
        let len = vd.length();
        if len <= delta || approx::abs_diff_eq!(0.0, len) {
            to
        } else {
            Self::linear_interpolate(self, to, delta / len)
        }
    }

    /// Returns the vector scaled to unit length. Equivalent to `v / v.length()`.
    #[inline]
    pub fn normalized(self) -> Self {
        Self::gd(self.glam().normalize())
    }

    /// Returns a vector composed of the @GDScript.fposmod of this vector's components and `rem`.
    #[inline]
    pub fn posmod(self, rem: f32) -> Self {
        self.posmodv(Self::new(rem, rem))
    }

    /// Returns a vector composed of the @GDScript.fposmod of this vector's components and `remv` components.
    #[inline]
    pub fn posmodv(self, remv: Self) -> Self {
        Self::new(self.x.rem_euclid(remv.x), self.y.rem_euclid(remv.y))
    }

    /// Returns the vector projected onto the vector `b`.
    #[inline]
    pub fn project(self, b: Self) -> Self {
        b * (self.dot(b) / b.length_squared())
    }

    /// Returns the vector reflected from a plane defined by the given normal.
    #[inline]
    pub fn reflect(self, n: Self) -> Self {
        n * self.dot(n) * 2.0 - self
    }

    /// Returns the vector rotated by `angle` radians.
    #[inline]
    pub fn rotated(self, angle: f32) -> Self {
        let (cos, sin) = (angle.cos(), angle.sin());
        Self::new(cos * self.x - sin * self.y, sin * self.x + cos * self.y)
    }

    /// Returns the vector with all components rounded to the nearest integer, with halfway cases
    /// rounded away from zero.
    #[inline]
    pub fn round(self) -> Self {
        Self::gd(self.glam().round())
    }

    /// Returns the vector with each component set to one or negative one, depending on the signs
    /// of the components, or zero if the component is zero, by calling @GDScript.sign on each
    /// component.
    #[inline]
    pub fn sign(self) -> Self {
        Self::gd(self.glam().signum())
    }

    /// Returns the result of spherical linear interpolation between this vector and b, by amount
    /// t. t is on the range of 0.0 to 1.0, representing the amount of interpolation.
    ///
    /// Note: Both vectors must be normalized.
    #[inline]
    pub fn slerp(self, b: Self, t: f32) -> Self {
        let theta = self.angle_to(b);
        self.rotated(theta * t)
    }

    /// Returns the component of the vector along a plane defined by the given normal.
    #[inline]
    pub fn slide(self, normal: Self) -> Self {
        self - normal * self.dot(normal)
    }

    /// Returns the vector snapped to a grid with the given size.
    #[inline]
    pub fn snapped(self, by: Self) -> Self {
        Vector2::new(
            if by.x != 0.0 {
                (self.x / by.x + 0.5).floor() * by.x
            } else {
                self.x
            },
            if by.y != 0.0 {
                (self.y / by.y + 0.5).floor() * by.y
            } else {
                self.y
            },
        )
    }

    /// Returns a perpendicular vector.
    #[inline]
    pub fn tangent(self) -> Self {
        Vector2::new(self.y, -self.x)
    }

    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub fn to_sys(self) -> sys::godot_vector2 {
        unsafe { std::mem::transmute(self) }
    }

    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_vector2 {
        self as *const _ as *const _
    }

    /// Internal API for converting from `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[inline]
    pub fn from_sys(v: sys::godot_vector2) -> Self {
        unsafe { std::mem::transmute(v) }
    }

    fn glam(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    fn gd(from: Vec2) -> Self {
        Self::new(from.x, from.y)
    }
}

macro_rules! derive_op_impl {
    ($trait:ident, $func:ident) => {
        impl $trait for Vector2 {
            type Output = Self;

            #[inline]
            fn $func(self, with: Self) -> Self {
                Self::gd(self.glam().$func(with.glam()))
            }
        }
    };
    ($trait:ident, $func:ident, $in_type:ty) => {
        impl $trait<$in_type> for Vector2 {
            type Output = Self;

            #[inline]
            fn $func(self, with: $in_type) -> Self {
                Self::gd(self.glam().$func(with))
            }
        }
    };
}

macro_rules! derive_assign_op_impl {
    ($trait:ident, $func:ident, $op_func:ident) => {
        impl $trait for Vector2 {
            #[inline]
            fn $func(&mut self, with: Self) {
                *self = self.$op_func(with);
            }
        }
    };
    ($trait:ident, $func:ident, $op_func:ident, $in_type:ty) => {
        impl $trait<$in_type> for Vector2 {
            #[inline]
            fn $func(&mut self, with: $in_type) {
                *self = self.$op_func(with);
            }
        }
    };
}

derive_op_impl!(Add, add);
derive_op_impl!(Sub, sub);
derive_op_impl!(Mul, mul);
derive_op_impl!(Div, div);
derive_op_impl!(Mul, mul, f32);
derive_op_impl!(Div, div, f32);
derive_assign_op_impl!(AddAssign, add_assign, add);
derive_assign_op_impl!(SubAssign, sub_assign, sub);
derive_assign_op_impl!(MulAssign, mul_assign, mul);
derive_assign_op_impl!(DivAssign, div_assign, div);
derive_assign_op_impl!(MulAssign, mul_assign, mul, f32);
derive_assign_op_impl!(DivAssign, div_assign, div, f32);

impl Neg for Vector2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::gd(-self.glam())
    }
}

godot_test!(
    test_vector2_variants {
        use crate::core_types::ToVariant;

        fn test(vector: Vector2, set_to: Vector2) {
            use crate::core_types::FromVariant;
            let api = crate::private::get_api();

            let copied = vector;
            unsafe {
                assert_relative_eq!(
                    vector.x,
                    (api.godot_vector2_get_x)(&copied as *const _ as *const sys::godot_vector2),
                );
                assert_relative_eq!(
                    vector.y,
                    (api.godot_vector2_get_y)(&copied as *const _ as *const sys::godot_vector2),
                );
            }
            assert_eq!(vector, copied);

            let mut copied = vector;
            unsafe {
                (api.godot_vector2_set_x)(&mut copied as *mut _ as *mut sys::godot_vector2, set_to.x);
                (api.godot_vector2_set_y)(&mut copied as *mut _ as *mut sys::godot_vector2, set_to.y);
            }
            assert_eq!(set_to, copied);

            let variant = vector.to_variant();
            let vector_from_variant = Vector2::from_variant(&variant).unwrap();
            assert_eq!(vector, vector_from_variant);
        }

        test(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0));
        test(Vector2::new(3.0, 4.0), Vector2::new(5.0, 6.0));
    }
);

#[cfg(test)]
mod tests {
    use crate::core_types::Vector2;

    #[test]
    fn it_is_copy() {
        fn copy<T: Copy>() {}
        copy::<Vector2>();
    }

    #[test]
    fn it_has_the_same_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<sys::godot_vector2>(), size_of::<Vector2>());
    }

    #[test]
    fn it_supports_equality() {
        assert_eq!(Vector2::new(1.0, 2.0), Vector2::new(1.0, 2.0));
    }

    #[test]
    fn it_supports_inequality() {
        assert_ne!(Vector2::new(1.0, 10.0), Vector2::new(1.0, 2.0));
    }

    #[test]
    fn cubic_interpolate_is_sane() {
        //use euclid::approxeq::ApproxEq;
        use Vector2 as V;

        assert!(
            V::new(4.7328, -6.7936).is_equal_approx(V::new(5.4, -6.8).cubic_interpolate(
                V::new(-1.2, 0.8),
                V::new(1.2, 10.3),
                V::new(-5.4, 4.2),
                0.2,
            ))
        );

        assert!(
            V::new(-3.8376, 2.9384).is_equal_approx(V::new(-4.2, 1.4).cubic_interpolate(
                V::new(-3.7, 2.1),
                V::new(5.4, -8.5),
                V::new(-10.8, -6.6),
                0.6,
            ))
        );
    }

    #[test]
    fn slide_is_sane() {
        use Vector2 as V;

        let cases = &[
            (V::new(1.0, 1.0), V::new(0.0, 1.0), V::new(1.0, 0.0)),
            (
                V::new(3.0, 4.0),
                V::new(-3.0, 1.0).normalized(),
                V::new(1.5, 4.5),
            ),
            (
                V::new(-2.0, 1.0),
                V::new(-1.0, 3.0).normalized(),
                V::new(-1.5, -0.5),
            ),
        ];

        for &(v, normal, expected) in cases {
            assert!(expected.is_equal_approx(v.slide(normal)));
        }
    }

    #[test]
    fn snapped_is_sane() {
        use Vector2 as V;

        let cases = &[
            (V::new(1.5, 5.6), V::new(1.0, 4.0), V::new(2.0, 4.0)),
            (V::new(5.4, 4.2), V::new(-2.0, -3.5), V::new(6.0, 3.5)),
            (V::new(5.4, -6.8), V::new(0.0, 0.3), V::new(5.4, -6.9)),
        ];

        for &(v, by, expected) in cases {
            assert!(expected.is_equal_approx(v.snapped(by)));
        }
    }
}

use super::geom::Basis;
use super::IsEqualApprox;
use glam::Vec3A;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 3D vector class.
///
/// See also [Vector3](https://docs.godotengine.org/en/stable/classes/class_vector3.html) in the Godot API doc.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[allow(clippy::unnecessary_cast)] // False positives: casts necessary for cross-platform
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Axis {
    X = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_X as u32,
    Y = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Y as u32,
    Z = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Z as u32,
}

impl Axis {
    /// Returns this axis as a vector of length 1, with only one component set.
    #[inline]
    pub fn to_unit_vector(self) -> Vector3 {
        match self {
            Axis::X => Vector3::RIGHT,
            Axis::Y => Vector3::UP,
            Axis::Z => Vector3::BACK,
        }
    }
}

/// Helper methods for `Vector3`.
///
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/3.1/classes/class_vector3.html).
impl Vector3 {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// A vector with all components set to 1. Typically used for scaling.
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0);

    /// A vector with all components set to +infinity.
    pub const INF: Self = Self::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);

    /// Unit vector in -X direction.
    pub const LEFT: Self = Self::new(-1.0, 0.0, 0.0);

    /// Unit vector in +X direction.
    pub const RIGHT: Self = Self::new(1.0, 0.0, 0.0);

    /// Unit vector in +Y direction.
    pub const UP: Self = Self::new(0.0, 1.0, 0.0);

    /// Unit vector in -Y direction.
    pub const DOWN: Self = Self::new(0.0, -1.0, 0.0);

    /// Unit vector in -Z direction.
    pub const FORWARD: Self = Self::new(0.0, 0.0, -1.0);

    /// Unit vector in +Z direction.
    pub const BACK: Self = Self::new(0.0, 0.0, 1.0);

    /// Returns a Vector3 with the given components.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns a new vector with all components in absolute values (i.e. positive).
    #[inline]
    pub fn abs(self) -> Self {
        Self::gd(self.glam().abs())
    }

    /// Returns the minimum angle to the given vector, in radians.
    #[inline]
    pub fn angle_to(self, to: Self) -> f32 {
        self.glam().angle_between(to.glam())
    }

    /// Returns the vector "bounced off" from a plane defined by the given normal.
    #[inline]
    pub fn bounce(self, n: Self) -> Self {
        -self.reflect(n)
    }

    /// Returns a new vector with all components rounded up (towards positive infinity).
    #[inline]
    pub fn ceil(self) -> Self {
        Self::gd(self.glam().ceil())
    }

    /// Returns the cross product of this vector and b.
    #[inline]
    pub fn cross(self, b: Self) -> Self {
        Self::gd(self.glam().cross(b.glam()))
    }

    /// Performs a cubic interpolation between vectors pre_a, a, b, post_b (a is current), by the
    /// given amount t. t is on the range of 0.0 to 1.0, representing the amount of interpolation.
    #[inline]
    pub fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self {
        let mut p = (pre_a, self, b, post_b);

        {
            let ab = p.0.distance_to(p.1);
            let bc = p.1.distance_to(p.2);
            let cd = p.2.distance_to(p.3);

            if ab > 0.0 {
                p.0 = p.1 + (p.0 - p.1) * (bc / ab);
            }
            if cd > 0.0 {
                p.3 = p.2 + (p.3 - p.2) * (bc / cd);
            }
        }

        let t = (t, t * t, t * t * t);

        0.5 * ((p.1 * 2.0)
            + (-p.0 + p.2) * t.0
            + (2.0 * p.0 - 5.0 * p.1 + 4.0 * p.2 - p.3) * t.1
            + (-p.0 + 3.0 * p.1 - 3.0 * p.2 + p.3) * t.2)
    }

    /// Returns the normalized vector pointing from this vector to `other`.
    #[inline]
    pub fn direction_to(self, other: Vector3) -> Vector3 {
        Self::gd((other.glam() - self.glam()).normalize())
    }

    /// Returns the squared distance to `other`.
    ///
    /// This method runs faster than distance_to, so prefer it if you need to compare vectors or
    /// need the squared distance for some formula.
    #[inline]
    pub fn distance_squared_to(self, other: Vector3) -> f32 {
        other.glam().distance_squared(self.glam())
    }

    /// Returns the distance to `other`.
    #[inline]
    pub fn distance_to(self, other: Vector3) -> f32 {
        other.glam().distance(self.glam())
    }

    /// Returns the dot product of this vector and b. This can be used to compare the angle between
    /// two vectors. For example, this can be used to determine whether an enemy is facing the player.
    ///
    /// The dot product will be 0 for a straight angle (90 degrees), greater than 0 for angles
    /// narrower than 90 degrees and lower than 0 for angles wider than 90 degrees.
    ///
    /// When using unit (normalized) vectors, the result will always be between -1.0 (180 degree
    /// angle) when the vectors are facing opposite directions, and 1.0 (0 degree angle) when the
    /// vectors are aligned.
    ///
    /// Note: a.dot(b) is equivalent to b.dot(a).
    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.glam().dot(other.glam())
    }

    /// Returns a new vector with all components rounded down (towards negative infinity).
    #[inline]
    pub fn floor(self) -> Self {
        Self::gd(self.glam().floor())
    }

    /// Returns the inverse of the vector. This is the same as
    /// `Vector3::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z)`.
    #[inline]
    pub fn inverse(self) -> Self {
        Self::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z)
    }

    /// Returns `true` if this vector and v are approximately equal, by running `relative_eq` on
    /// each component.
    #[inline]
    pub fn is_equal_approx(self, v: Self) -> bool {
        self.x.is_equal_approx(v.x) && self.y.is_equal_approx(v.y) && self.z.is_equal_approx(v.z)
    }

    /// Returns `true` if the vector is normalized, and `false` otherwise.
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

    /// Returns the axis of the vector's largest value. See [`Axis`] enum.
    ///
    /// If multiple components are equal, this method returns in preferred order `Axis::X`, `Axis::Y`, `Axis::Z`.
    #[inline]
    #[allow(clippy::collapsible_else_if)]
    pub fn max_axis(self) -> Axis {
        if self.z > self.y {
            if self.z > self.x {
                Axis::Z
            } else {
                Axis::X
            }
        } else {
            if self.y > self.x {
                Axis::Y
            } else {
                Axis::X
            }
        }
    }

    /// Returns the axis of the vector's smallest value. See `Axis` enum.
    ///
    /// If multiple components are equal, this method returns in preferred order `Axis::X`, `Axis::Y`, `Axis::Z`.
    #[inline]
    #[allow(clippy::collapsible_else_if)]
    pub fn min_axis(self) -> Axis {
        if self.x < self.y {
            if self.x < self.z {
                Axis::X
            } else {
                Axis::Z
            }
        } else {
            if self.y < self.z {
                Axis::Y
            } else {
                Axis::Z
            }
        }
    }

    /// Moves this vector toward `to` by the fixed `delta` amount.
    #[inline]
    pub fn move_toward(self, to: Self, delta: f32) -> Self {
        let vd = to - self;
        let len = vd.length();
        if len <= delta || approx::abs_diff_eq!(0.0, len) {
            to
        } else {
            self.linear_interpolate(to, delta / len)
        }
    }

    /// Returns the vector scaled to unit length. Equivalent to `v / v.length()`.
    #[inline]
    pub fn normalized(self) -> Self {
        Self::gd(self.glam().normalize())
    }

    /// Returns the outer product with `b`.
    #[inline]
    pub fn outer(self, b: Self) -> Basis {
        Basis::from_rows(b * self.x, b * self.y, b * self.z)
    }

    /// Returns a vector composed of the `rem_euclid` of this vector's components and `mod`.
    #[inline]
    pub fn posmod(self, rem: f32) -> Self {
        self.posmodv(Self::new(rem, rem, rem))
    }

    /// Returns a vector composed of the `rem_euclid` of this vector's components and `remv`
    /// components.
    #[inline]
    pub fn posmodv(self, remv: Self) -> Self {
        Self::new(
            self.x.rem_euclid(remv.x),
            self.y.rem_euclid(remv.y),
            self.z.rem_euclid(remv.z),
        )
    }

    /// Returns this vector projected onto another vector `b`.
    #[inline]
    pub fn project(self, b: Self) -> Self {
        b * (self.dot(b) / b.length_squared())
    }

    /// Returns this vector reflected from a plane defined by the given normal.
    #[inline]
    pub fn reflect(self, n: Self) -> Self {
        n * self.dot(n) * 2.0 - self
    }

    /// Rotates this vector around a given axis by `phi` radians. The axis must be a normalized
    /// vector.
    #[inline]
    pub fn rotated(self, axis: Self, phi: f32) -> Self {
        Basis::from_axis_angle(axis, phi) * self
    }

    /// Returns this vector with all components rounded to the nearest integer, with halfway cases
    /// rounded away from zero.
    #[inline]
    pub fn round(self) -> Self {
        Self::gd(self.glam().round())
    }

    /// Returns a vector with each component set to one or negative one, depending on the signs of
    /// this vector's components, or zero if the component is zero, by calling `signum` on each
    /// component.
    #[inline]
    pub fn sign(self) -> Self {
        Self::gd(self.glam().signum())
    }

    /// Returns the result of spherical linear interpolation between this vector and b, by amount t.
    /// t is on the range of 0.0 to 1.0, representing the amount of interpolation.
    ///
    /// **Note**: Both vectors must be normalized.
    #[inline]
    pub fn slerp(self, b: Self, t: f32) -> Self {
        let theta = self.angle_to(b);
        self.rotated(self.cross(b).normalized(), theta * t)
    }

    /// Returns this vector slid along a plane defined by the given normal.
    #[inline]
    pub fn slide(self, n: Self) -> Self {
        self - n * self.dot(n)
    }

    /// Returns this vector with each component snapped to the nearest multiple of step.
    /// This can also be used to round to an arbitrary number of decimals.
    #[inline]
    pub fn snapped(self, by: Self) -> Self {
        let stepify = |v: f32, s: f32| {
            if by.x != 0.0 {
                (v / s + 0.5).floor() * s
            } else {
                v
            }
        };
        Self::new(
            stepify(self.x, by.x),
            stepify(self.y, by.y),
            stepify(self.z, by.z),
        )
    }

    /// Returns a diagonal matrix with the vector as main diagonal.
    ///
    /// This is equivalent to a Basis with no rotation or shearing and this vector's components set
    /// as the scale.
    #[inline]
    pub fn to_diagonal_matrix(self) -> Basis {
        Basis::from_diagonal(self)
    }

    #[doc(hidden)]
    #[allow(clippy::wrong_self_convention)]
    #[inline]
    pub fn to_sys(self) -> sys::godot_vector3 {
        unsafe { std::mem::transmute(self) }
    }

    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_vector3 {
        self as *const _ as *const _
    }

    /// Internal API for converting from `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[inline]
    pub fn from_sys(v: sys::godot_vector3) -> Self {
        unsafe { std::mem::transmute(v) }
    }

    #[inline]
    pub(super) fn glam(self) -> Vec3A {
        Vec3A::new(self.x, self.y, self.z)
    }

    #[inline]
    pub(super) fn gd(from: Vec3A) -> Self {
        Self::new(from.x, from.y, from.z)
    }
}

impl AsRef<[f32; 3]> for Vector3 {
    #[inline]
    fn as_ref(&self) -> &[f32; 3] {
        // SAFETY: Vector3 is repr(C)
        unsafe { &*(self as *const Vector3 as *const [f32; 3]) }
    }
}

macro_rules! derive_op_impl {
    ($trait:ident, $func:ident) => {
        impl $trait for Vector3 {
            type Output = Self;

            #[inline]
            fn $func(self, with: Self) -> Self {
                Self::gd(self.glam().$func(with.glam()))
            }
        }
    };
    ($trait:ident, $func:ident, $in_type:ty) => {
        impl $trait<$in_type> for Vector3 {
            type Output = Self;

            #[inline]
            fn $func(self, with: $in_type) -> Self {
                Self::gd(self.glam().$func(with))
            }
        }
    };
}

macro_rules! derive_op_impl_rev {
    ($trait:ident, $func:ident, $in_type:ty) => {
        impl $trait<Vector3> for $in_type {
            type Output = Vector3;

            #[inline]
            fn $func(self, with: Self::Output) -> Self::Output {
                $trait::$func(with, self)
            }
        }
    };
}

macro_rules! derive_assign_op_impl {
    ($trait:ident, $func:ident, $op_func:ident) => {
        impl $trait for Vector3 {
            #[inline]
            fn $func(&mut self, with: Self) {
                *self = self.$op_func(with);
            }
        }
    };
    ($trait:ident, $func:ident, $op_func:ident, $in_type:ty) => {
        impl $trait<$in_type> for Vector3 {
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
derive_op_impl_rev!(Mul, mul, f32);
derive_assign_op_impl!(AddAssign, add_assign, add);
derive_assign_op_impl!(SubAssign, sub_assign, sub);
derive_assign_op_impl!(MulAssign, mul_assign, mul);
derive_assign_op_impl!(DivAssign, div_assign, div);
derive_assign_op_impl!(MulAssign, mul_assign, mul, f32);
derive_assign_op_impl!(DivAssign, div_assign, div, f32);

impl Neg for Vector3 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::gd(-self.glam())
    }
}

godot_test!(
    test_vector3_variants {
        use crate::core_types::{FromVariant, ToVariant, Vector3};

        fn test(vector: Vector3, set_to: Vector3) {
            let api = crate::private::get_api();

            let copied = vector;
            unsafe {
                assert_relative_eq!(vector.x, (api.godot_vector3_get_axis)(
                    &copied as *const _ as *const sys::godot_vector3,
                    Axis::X as u32 as sys::godot_vector3_axis
                ));
                assert_relative_eq!(vector.y, (api.godot_vector3_get_axis)(
                    &copied as *const _ as *const sys::godot_vector3,
                    Axis::Y as u32 as sys::godot_vector3_axis
                ));
                assert_relative_eq!(vector.z, (api.godot_vector3_get_axis)(
                    &copied as *const _ as *const sys::godot_vector3,
                    Axis::Z as u32 as sys::godot_vector3_axis
                ));
            }
            assert_eq!(vector, copied);

            let mut copied = vector;
            unsafe {
                (api.godot_vector3_set_axis)(
                    &mut copied as *mut _ as *mut sys::godot_vector3,
                    Axis::X as u32 as sys::godot_vector3_axis,
                    set_to.x
                );
                (api.godot_vector3_set_axis)(
                    &mut copied as *mut _ as *mut sys::godot_vector3,
                    Axis::Y as u32 as sys::godot_vector3_axis,
                    set_to.y
                );
                (api.godot_vector3_set_axis)(
                    &mut copied as *mut _ as *mut sys::godot_vector3,
                    Axis::Z as u32 as sys::godot_vector3_axis,
                    set_to.z
                );
            }
            assert_eq!(set_to, copied);

            let variant = vector.to_variant();
            let vector_from_variant = Vector3::from_variant(&variant).unwrap();
            assert_eq!(vector, vector_from_variant);
        }

        test(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0));
        test(Vector3::new(4.0, 5.0, 6.0), Vector3::new(7.0, 8.0, 9.0));
    }
);

#[cfg(test)]
mod tests {
    use crate::core_types::Vector3;

    /*
     * Test introduced due to bug in Basis * Vector3 operator
     *
     * matching result in GDScript:
     * var v1 = Vector3(37.51756, 20.39467, 49.96816)
     * var phi = -0.4927880786382844
     * var v2 = v1.rotated(Vector3.UP, r)
     * print(c)
     */
    #[test]
    fn rotated() {
        let v = Vector3::new(37.51756, 20.39467, 49.96816);
        let phi = -0.4927880786382844;
        let r = v.rotated(Vector3::UP, phi);
        assert!(r.is_equal_approx(Vector3::new(9.414476, 20.39467, 61.77177)));
    }

    #[test]
    fn it_is_copy() {
        fn copy<T: Copy>() {}
        copy::<Vector3>();
    }

    #[test]
    fn it_has_the_same_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<sys::godot_vector3>(), size_of::<Vector3>());
    }

    #[test]
    fn it_supports_equality() {
        assert_eq!(Vector3::new(1.0, 2.0, 3.0), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn it_supports_inequality() {
        assert_ne!(Vector3::new(1.0, 10.0, 100.0), Vector3::new(1.0, 2.0, 3.0));
    }
}

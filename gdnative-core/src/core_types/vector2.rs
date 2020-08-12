use crate::core_types::{Angle, Rotation2D, Vector2};

/// Helper methods for `Vector2`.
///
/// Trait used to provide additional methods that are equivalent to Godot's methods.
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/3.1/classes/class_vector2.html).
pub trait Vector2Godot {
    /// Returns the ratio of x to y.
    fn aspect(self) -> f32;
    /// Cubicly interpolates between this vector and `b` using `pre_a` and `post_b` as handles,
    /// and returns the result at position `t`. `t` is in the range of 0.0 - 1.0, representing
    /// the amount of interpolation.
    fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self;
    /// Returns the vector rotated by `angle` radians.
    fn rotated(self, angle: Angle) -> Self;
    /// Returns the component of the vector along a plane defined by the given normal.
    fn slide(self, normal: Self) -> Self;
    /// Returns the vector snapped to a grid with the given size.
    fn snapped(self, by: Self) -> Self;
    /// Returns a perpendicular vector.
    fn tangent(self) -> Self;
    /// Returns `self` moved towards `to` by the distance `delta`, clamped by `to`.
    fn move_towards(self, to: Vector2, delta: f32) -> Self;
    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    fn to_sys(self) -> sys::godot_vector2;
    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    fn sys(&self) -> *const sys::godot_vector2;
    /// Internal API for converting from `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    fn from_sys(v: sys::godot_vector2) -> Self;
}

impl Vector2Godot for Vector2 {
    #[inline]
    fn aspect(self) -> f32 {
        self.x / self.y
    }

    #[inline]
    fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self {
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

    #[inline]
    fn rotated(self, angle: Angle) -> Self {
        let r = Rotation2D::new(angle);
        r.transform_vector(self)
    }

    #[inline]
    fn slide(self, normal: Self) -> Self {
        self - normal * self.dot(normal)
    }

    #[inline]
    fn snapped(self, by: Self) -> Self {
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

    #[inline]
    fn tangent(self) -> Self {
        Vector2::new(self.y, -self.x)
    }

    #[inline]
    fn move_towards(self, to: Vector2, delta: f32) -> Self {
        let vd = to - self;
        let len = vd.length();
        if len <= delta || approx::abs_diff_eq!(0.0, len) {
            to
        } else {
            Vector2::lerp(&self, to, delta / len)
        }
    }

    #[inline]
    fn to_sys(self) -> sys::godot_vector2 {
        unsafe { std::mem::transmute(self) }
    }

    #[inline]
    fn sys(&self) -> *const sys::godot_vector2 {
        self as *const _ as *const _
    }

    #[inline]
    fn from_sys(v: sys::godot_vector2) -> Self {
        unsafe { std::mem::transmute(v) }
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
    use crate::core_types::vector2::Vector2Godot;
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
        use euclid::approxeq::ApproxEq;
        use Vector2 as V;

        assert!(
            V::new(4.7328, -6.7936).approx_eq(&V::new(5.4, -6.8).cubic_interpolate(
                V::new(-1.2, 0.8),
                V::new(1.2, 10.3),
                V::new(-5.4, 4.2),
                0.2
            ))
        );

        assert!(
            V::new(-3.8376, 2.9384).approx_eq(&V::new(-4.2, 1.4).cubic_interpolate(
                V::new(-3.7, 2.1),
                V::new(5.4, -8.5),
                V::new(-10.8, -6.6),
                0.6
            ))
        );
    }

    #[test]
    fn slide_is_sane() {
        use euclid::approxeq::ApproxEq;
        use Vector2 as V;

        let cases = &[
            (V::new(1.0, 1.0), V::new(0.0, 1.0), V::new(1.0, 0.0)),
            (
                V::new(3.0, 4.0),
                V::new(-3.0, 1.0).normalize(),
                V::new(1.5, 4.5),
            ),
            (
                V::new(-2.0, 1.0),
                V::new(-1.0, 3.0).normalize(),
                V::new(-1.5, -0.5),
            ),
        ];

        for &(v, normal, expected) in cases {
            assert!(expected.approx_eq(&v.slide(normal)));
        }
    }

    #[test]
    fn snapped_is_sane() {
        use euclid::approxeq::ApproxEq;
        use Vector2 as V;

        let cases = &[
            (V::new(1.5, 5.6), V::new(1.0, 4.0), V::new(2.0, 4.0)),
            (V::new(5.4, 4.2), V::new(-2.0, -3.5), V::new(6.0, 3.5)),
            (V::new(5.4, -6.8), V::new(0.0, 0.3), V::new(5.4, -6.9)),
        ];

        for &(v, by, expected) in cases {
            assert!(expected.approx_eq(&v.snapped(by)));
        }
    }
}

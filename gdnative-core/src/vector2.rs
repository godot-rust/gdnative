use crate::FromVariant;
use crate::ToVariant;
use crate::Variant;
use crate::Vector2;
use euclid::{Angle, Length, Point2D, Rotation2D, Trig, UnknownUnit};

impl ToVariant for Vector2 {
    fn to_variant(&self) -> Variant {
        Variant::from_vector2(self)
    }
}

impl FromVariant for Vector2 {
    fn from_variant(variant: &Variant) -> Option<Self> {
        variant.try_to_vector2()
    }
}

/// Helper methods for `Vector2`.
///
/// Trait used to provide additional methods that are equivalent to Godot's methods.
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/3.1/classes/class_vector2.html).
pub trait Vector2Godot {
    /// Returns the angle in radians between the two vectors.
    fn angle_to(self, other: Self) -> Angle<f32>;
    /// Returns the angle in radians between the line connecting the two points and the x
    /// coordinate.
    fn angle_to_point(self, point: Point2D<f32, UnknownUnit>) -> Angle<f32>;
    /// Returns the ratio of x to y.
    fn aspect(self) -> f32;
    /// Returns the vector “bounced off” from a plane defined by the given normal.
    fn bounce(self, normal: Self) -> Self;
    /// Returns the vector with a maximum length.
    fn clamped(self, length: f32) -> Self;
    /// Cubicly interpolates between this vector and `b` using `pre_a` and `post_b` as handles,
    /// and returns the result at position `t`. `t` is in the range of 0.0 - 1.0, representing
    /// the amount of interpolation.
    fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self;
    /// Returns the normalized vector pointing from this vector to `point`.
    fn direction_to(self, point: Point2D<f32, UnknownUnit>) -> Self;
    /// Returns the distance to `point`.
    fn distance_to(self, point: Point2D<f32, UnknownUnit>) -> Length<f32, UnknownUnit>;
    /// Returns the squared distance to `point`. Prefer this function over distance_to if you
    /// need to sort vectors or need the squared distance for some formula.
    fn distance_squared_to(self, point: Point2D<f32, UnknownUnit>) -> Length<f32, UnknownUnit>;
    /// Returns the vector projected onto the `other` vector.
    fn project(self, other: Self) -> Self;
    /// Returns the vector reflected from a plane defined by the given `normal`.
    fn reflect(self, normal: Self) -> Self;
    /// Returns the vector rotated by `angle` radians.
    fn rotated(self, angle: Angle<f32>) -> Self;
    /// Returns the component of the vector along a plane defined by the given normal.
    fn slide(self, normal: Self) -> Self;
    /// Returns the vector snapped to a grid with the given size.
    fn snapped(self, by: Self) -> Self;
    /// Returns a perpendicular vector.
    fn tangent(self) -> Self;
}

impl Vector2Godot for Vector2 {
    #[inline]
    fn angle_to(self, other: Self) -> Angle<f32> {
        Angle::radians(Trig::fast_atan2(self.cross(other), self.dot(other)))
    }

    #[inline]
    fn angle_to_point(self, point: Point2D<f32, UnknownUnit>) -> Angle<f32> {
        Angle::radians(Trig::fast_atan2(self.y - point.y, self.x - point.x))
    }

    #[inline]
    fn aspect(self) -> f32 {
        self.x / self.y
    }

    #[inline]
    fn bounce(self, normal: Self) -> Self {
        let normal = normal.normalize();
        normal * ((-2.0) * self.dot(normal)) + self
    }

    #[inline]
    fn clamped(self, length: f32) -> Self {
        if self.length() > length {
            self.normalize() * length
        } else {
            self
        }
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
    fn direction_to(self, point: Point2D<f32, UnknownUnit>) -> Self {
        (point - self).to_vector().normalize()
    }

    #[inline]
    fn distance_to(self, point: Point2D<f32, UnknownUnit>) -> Length<f32, UnknownUnit> {
        let squared = (self.x - point.x).powi(2) + (self.y - point.y).powi(2);
        Length::new(squared.sqrt())
    }

    #[inline]
    fn distance_squared_to(self, point: Point2D<f32, UnknownUnit>) -> Length<f32, UnknownUnit> {
        Length::new((self.x - point.x).powi(2) + (self.y - point.y).powi(2))
    }

    #[inline]
    fn project(self, other: Self) -> Self {
        let v1 = other;
        let v2 = self;
        v2 * (v1.dot(v2) / v2.dot(v2))
    }

    #[inline]
    fn reflect(self, normal: Self) -> Self {
        normal - self * self.dot(normal) * 2.0
    }

    #[inline]
    fn rotated(self, angle: Angle<f32>) -> Self {
        let r = Rotation2D::new(angle);
        r.transform_vector(self)
    }

    #[inline]
    fn slide(self, normal: Self) -> Self {
        normal - self * self.dot(normal)
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
}

godot_test!(
    test_vector2_variants {
        fn test(vector: Vector2, set_to: Vector2) {
            let api = crate::get_api();

            let copied = vector;
            unsafe {
                assert_eq!(vector.x, (api.godot_vector2_get_x)(&copied as *const _ as *const sys::godot_vector2));
                assert_eq!(vector.y, (api.godot_vector2_get_y)(&copied as *const _ as *const sys::godot_vector2));
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
    use super::Vector2;

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
}

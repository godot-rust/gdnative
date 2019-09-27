use crate::FromVariant;
use crate::ToVariant;
use crate::Variant;
use crate::Vector2;
use euclid::{Angle, Length, Rotation2D, Trig, UnknownUnit};

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
    fn angle(self) -> Angle<f32>;
    fn angle_to(self, other: Self) -> Angle<f32>;
    fn angle_to_point(self, other: Self) -> Angle<f32>;
    fn aspect(self) -> f32;
    fn bounce(self, normal: Self) -> Self;
    fn clamped(self, length: f32) -> Self;
    fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, t: f32) -> Self;
    fn direction_to(self, other: Self) -> Self;
    fn distance_to(self, other: Self) -> Length<f32, UnknownUnit>;
    fn distance_squared_to(self, other: Self) -> Length<f32, UnknownUnit>;
    fn project(self, other: Self) -> Self;
    fn reflect(self, other: Self) -> Self;
    fn rotated(self, angle: Angle<f32>) -> Self;
    fn slide(self, other: Self) -> Self;
    fn snapped(self, other: Self) -> Self;
    fn tangent(self) -> Self;
}

impl Vector2Godot for Vector2 {
    #[inline]
    fn angle(self) -> Angle<f32> {
        self.angle_from_x_axis()
    }

    #[inline]
    fn angle_to(self, other: Self) -> Angle<f32> {
        Angle::radians(Trig::fast_atan2(self.cross(other), self.dot(other)))
    }

    #[inline]
    fn angle_to_point(self, other: Self) -> Angle<f32> {
        Angle::radians(Trig::fast_atan2(self.y - other.y, self.x - other.x))
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
    fn direction_to(self, other: Self) -> Self {
        (other - self).normalize()
    }

    #[inline]
    fn distance_to(self, other: Self) -> Length<f32, UnknownUnit> {
        Length::new((self - other).length())
    }

    #[inline]
    fn distance_squared_to(self, other: Self) -> Length<f32, UnknownUnit> {
        Length::new((self - other).square_length())
    }

    #[inline]
    fn project(self, other: Self) -> Self {
        let v1 = other;
        let v2 = self;
        v2 * (v1.dot(v2) / v2.dot(v2))
    }

    #[inline]
    fn reflect(self, other: Self) -> Self {
        other - self * self.dot(other) * 2.0
    }

    #[inline]
    fn rotated(self, angle: Angle<f32>) -> Self {
        let r = Rotation2D::new(angle);
        r.transform_vector(self)
    }

    #[inline]
    fn slide(self, other: Self) -> Self {
        other - self * self.dot(other)
    }

    #[inline]
    fn snapped(self, other: Self) -> Self {
        Vector2::new(
            if other.x != 0.0 {
                (self.x / other.x + 0.5).floor() * other.x
            } else {
                self.x
            },
            if other.y != 0.0 {
                (self.y / other.y + 0.5).floor() * other.y
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

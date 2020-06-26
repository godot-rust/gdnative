use crate::core_types::{Angle, Point2, Vector2};
use euclid::Trig;

/// Helper methods for `Point2`.
///
/// Trait used to provide additional methods that are equivalent to Godot's methods.
/// See the official [`Godot documentation`](https://docs.godotengine.org/en/3.1/classes/class_vector2.html).
pub trait Point2Godot {
    /// Returns the angle in radians between the line connecting the two points and the x
    /// coordinate.
    fn angle_to_point(self, other: Point2) -> Angle;
    /// Returns the normalized vector pointing from this point to `other`.
    fn direction_to(self, other: Point2) -> Vector2;
    /// Returns the distance to `other`.
    fn distance_to(self, other: Point2) -> f32;
    /// Returns the squared distance to `other`. Prefer this function over distance_to if you
    /// need to sort points or need the squared distance for some formula.
    fn distance_squared_to(self, other: Point2) -> f32;
}

impl Point2Godot for Point2 {
    #[inline]
    fn angle_to_point(self, other: Point2) -> Angle {
        Angle::radians(Trig::fast_atan2(self.y - other.y, self.x - other.x))
    }

    #[inline]
    fn direction_to(self, other: Point2) -> Vector2 {
        (other - self).normalize()
    }

    #[inline]
    fn distance_to(self, other: Point2) -> f32 {
        (other - self).length()
    }

    #[inline]
    fn distance_squared_to(self, other: Point2) -> f32 {
        (other - self).square_length()
    }
}

use crate::core_types::{Axis, Plane, Vector3};

/// Axis-aligned bounding box.
///
/// `Aabb` consists of a position, a size, and several utility functions. It is typically used for
/// fast overlap tests.
///
/// The 2D counterpart to `Aabb` is [`Rect2`](crate::core_types::Rect2).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aabb {
    /// The bounding box's position in 3D space.
    pub position: Vector3,
    /// Width, height, and depth of the bounding box.
    pub size: Vector3,
}

impl Aabb {
    /// Creates an `Aabb` by position and size.
    #[inline]
    pub fn new(position: Vector3, size: Vector3) -> Self {
        Self { position, size }
    }

    /// Creates an `Aabb` by x, y, z, width, height, and depth.
    #[inline]
    pub fn from_components(x: f32, y: f32, z: f32, width: f32, height: f32, depth: f32) -> Self {
        let position = Vector3::new(x, y, z);
        let size = Vector3::new(width, height, depth);

        Self { position, size }
    }

    /// Ending corner. This is calculated as `position + size`.
    #[inline]
    pub fn get_end(&self) -> Vector3 {
        self.position + self.size
    }

    /// Ending corner. Setting this value will change the size.
    #[inline]
    pub fn set_end(&mut self, new_end: Vector3) {
        self.size = new_end - self.position;
    }

    /// Returns an `Aabb` with equivalent position and area, modified so that the most-negative
    /// corner is the origin and the size is positive.
    #[inline]
    pub fn abs(&self) -> Self {
        let position = self.position
            + Vector3::new(
                self.size.x.min(0.0),
                self.size.y.min(0.0),
                self.size.z.min(0.0),
            );
        let size = self.size.abs();

        Self { position, size }
    }

    /// Returns the volume of the bounding box. See also [`has_no_volume`][Self::has_no_volume].
    ///
    /// This method corresponds to the [`get_area`][get_area] GDScript method.
    ///
    /// [get_area]: https://docs.godotengine.org/en/stable/classes/class_aabb.html#class-aabb-method-get-area
    #[inline]
    pub fn get_volume(&self) -> f32 {
        self.size.x * self.size.y * self.size.z
    }

    /// Returns true if the bounding box is flat or empty. See also
    /// [`get_volume`][Self::get_volume].
    ///
    /// This method corresponds to the [`has_no_area`][has_no_area] GDScript method.
    ///
    /// Note: If the bounding box has a negative size and is not flat or empty, this method will
    /// return true.
    ///
    /// [has_no_area]: https://docs.godotengine.org/en/stable/classes/class_aabb.html#class-aabb-method-has-no-area
    #[inline]
    pub fn has_no_volume(&self) -> bool {
        self.size.x <= 0.0 || self.size.y <= 0.0 || self.size.z <= 0.0
    }

    /// Returns true if the bounding box is empty or all of its dimensions are negative.
    #[inline]
    pub fn has_no_surface(&self) -> bool {
        self.size.x <= 0.0 && self.size.y <= 0.0 && self.size.z <= 0.0
    }

    /// Returns true if the bounding box contains a point. By convention, the right and bottom edges of
    /// the `Rect2` are considered exclusive, so points on these edges are not included.
    ///
    /// Note: This method is not reliable for bounding boxes with a negative size. Use
    /// [`abs`][Self::abs] to get a positive sized equivalent box to check for contained points.
    #[inline]
    pub fn has_point(&self, point: Vector3) -> bool {
        let point = point - self.position;

        point.abs() == point
            && point.x < self.size.x
            && point.y < self.size.y
            && point.z < self.size.z
    }

    /// Returns true if this bounding box and `b` are approximately equal, by calling
    /// [`is_equal_approx`](Vector3::is_equal_approx) on each component.
    #[inline]
    pub fn is_equal_approx(&self, b: Self) -> bool {
        self.position.is_equal_approx(b.position) && self.size.is_equal_approx(b.size)
    }

    /// Gets the position of the 8 endpoints of the bounding in space.
    ///
    /// The index returns an arbitrary point, but all points are guaranteed to be unique.
    #[inline]
    pub fn get_endpoint(&self, index: i64) -> Option<Vector3> {
        match index {
            0 => Some(self.position),
            1 => Some(self.position + Vector3::new(0.0, 0.0, self.size.z)),
            2 => Some(self.position + Vector3::new(0.0, self.size.y, 0.0)),
            3 => Some(self.position + Vector3::new(0.0, self.size.y, self.size.z)),
            4 => Some(self.position + Vector3::new(self.size.x, 0.0, 0.0)),
            5 => Some(self.position + Vector3::new(self.size.x, 0.0, self.size.z)),
            6 => Some(self.position + Vector3::new(self.size.x, self.size.y, 0.0)),
            7 => Some(self.position + self.size),
            _ => None,
        }
    }

    /// Returns the normalized longest axis of the bounding box.
    #[inline]
    pub fn get_longest_axis(&self) -> Vector3 {
        let mut axis = Vector3::RIGHT;
        let mut axis_max = self.size.x;

        if self.size.y > axis_max {
            axis = Vector3::UP;
            axis_max = self.size.y;
        }

        if self.size.z > axis_max {
            axis = Vector3::BACK;
        }

        axis
    }

    /// Returns the index of the longest axis of the bounding box.
    #[inline]
    pub fn get_longest_axis_index(&self) -> Axis {
        let mut axis = Axis::X;
        let mut axis_max = self.size.x;

        if self.size.y > axis_max {
            axis = Axis::Y;
            axis_max = self.size.y;
        }

        if self.size.z > axis_max {
            axis = Axis::Z;
        }

        axis
    }

    /// Returns the scalar length of the longest axis of the bounding box.
    #[inline]
    pub fn get_longest_axis_size(&self) -> f32 {
        let mut axis_max = self.size.x;

        if self.size.y > axis_max {
            axis_max = self.size.y;
        }

        if self.size.z > axis_max {
            axis_max = self.size.z;
        }

        axis_max
    }

    /// Returns the normalized shortest axis of the bounding box.
    #[inline]
    pub fn get_shortest_axis(&self) -> Vector3 {
        let mut axis = Vector3::RIGHT;
        let mut axis_min = self.size.x;

        if self.size.y < axis_min {
            axis = Vector3::UP;
            axis_min = self.size.y;
        }

        if self.size.z < axis_min {
            axis = Vector3::BACK;
        }

        axis
    }

    /// Returns the index of the shortest axis of the bounding box.
    #[inline]
    pub fn get_shortest_axis_index(&self) -> Axis {
        let mut axis = Axis::X;
        let mut axis_min = self.size.x;

        if self.size.y < axis_min {
            axis = Axis::Y;
            axis_min = self.size.y;
        }

        if self.size.z < axis_min {
            axis = Axis::Z;
        }

        axis
    }

    /// Returns the scalar length of the shortest axis of the bounding box.
    #[inline]
    pub fn get_shortest_axis_size(&self) -> f32 {
        let mut axis_min = self.size.x;

        if self.size.y < axis_min {
            axis_min = self.size.y;
        }

        if self.size.z < axis_min {
            axis_min = self.size.z;
        }

        axis_min
    }

    /// Returns the support point in a given direction. This is useful for collision detection
    /// algorithms.
    #[inline]
    pub fn get_support(&self, dir: Vector3) -> Vector3 {
        let center = self.size * 0.5;
        let offset = self.position + center;

        Vector3::new(
            if dir.x > 0.0 { -center.x } else { center.x },
            if dir.y > 0.0 { -center.y } else { center.y },
            if dir.z > 0.0 { -center.z } else { center.z },
        ) + offset
    }

    /// Returns a copy of the bounding box grown a given amount of units on all the sides.
    #[inline]
    pub fn grow(&self, by: f32) -> Self {
        let position = self.position - Vector3::new(by, by, by);
        let size = self.size + Vector3::new(by, by, by) * 2.0;

        Self { position, size }
    }

    /// Returns true if the bounding box overlaps with `b`.
    #[inline]
    pub fn intersects(&self, b: Self) -> bool {
        !(self.position.x >= b.position.x + b.size.x
            || self.position.x + self.size.x <= b.position.x
            || self.position.y >= b.position.y + b.size.y
            || self.position.y + self.size.y <= b.size.y
            || self.position.z >= b.position.z + b.size.z
            || self.position.z + self.size.z <= b.size.z)
    }

    /// Returns true if the bounding box is on both sides of a plane.
    #[inline]
    pub fn intersects_plane(&self, plane: Plane) -> bool {
        let points = [
            self.position,
            self.position + Vector3::new(0.0, 0.0, self.size.z),
            self.position + Vector3::new(0.0, self.size.y, 0.0),
            self.position + Vector3::new(0.0, self.size.y, self.size.z),
            self.position + Vector3::new(self.size.x, 0.0, 0.0),
            self.position + Vector3::new(self.size.x, 0.0, self.size.z),
            self.position + Vector3::new(self.size.x, self.size.y, 0.0),
            self.position + self.size,
        ];

        let mut over = false;
        let mut under = false;

        for point in points {
            if plane.distance_to(point) > 0.0 {
                over = true;
            } else {
                under = true;
            }
        }

        under && over
    }

    /// Returns true if the bounding box intersects the line segment between `from` and `to`.
    #[inline]
    pub fn intersects_segment(&self, from: Vector3, to: Vector3) -> bool {
        let mut min: f32 = 0.0;
        let mut max: f32 = 1.0;

        let from = from.as_ref().iter();
        let to = to.as_ref().iter();
        let begin = self.position.as_ref().iter();
        let end = self.get_end();
        let end = end.as_ref().iter();

        for (((from, to), begin), end) in from.zip(to).zip(begin).zip(end) {
            let length = to - from;
            let mut cmin = 0.0;
            let mut cmax = 1.0;

            if from < to {
                if from > end || to < begin {
                    return false;
                }

                if from < begin {
                    cmin = (begin - from) / length;
                }
                if to < end {
                    cmax = (end - from) / length;
                }
            } else {
                if to > end || from < begin {
                    return false;
                }

                if from > end {
                    cmin = (end - from) / length;
                }
                if to < begin {
                    cmax = (begin - from) / length;
                }
            }

            min = min.max(cmin);
            max = max.min(cmax);
            if max < min {
                return false;
            }
        }

        true
    }

    /// Returns the intersection between two bounding boxes. An empty bounding box (size 0,0,0) is
    /// returned if there is no intersection.
    #[inline]
    pub fn intersection(&self, b: Self) -> Self {
        if !self.intersects(b) {
            return Self::default();
        }

        let mut aabb = b;
        aabb.position.x = aabb.position.x.max(self.position.x);
        aabb.position.y = aabb.position.y.max(self.position.y);
        aabb.position.z = aabb.position.z.max(self.position.z);

        let end = self.get_end();
        let end_b = b.get_end();

        aabb.size.x = end.x.min(end_b.x) - aabb.position.x;
        aabb.size.y = end.y.min(end_b.y) - aabb.position.y;
        aabb.size.z = end.y.min(end_b.z) - aabb.position.z;

        aabb
    }

    /// Returns a larger bounding box that contains both this `Aabb` and `b`.
    #[inline]
    pub fn merge(&self, b: Self) -> Self {
        let position = Vector3::new(
            self.position.x.min(b.position.x),
            self.position.y.min(b.position.y),
            self.position.z.min(b.position.z),
        );
        let end = Vector3::new(
            (self.position.x + self.size.x).max(b.position.x + b.size.x),
            (self.position.y + self.size.y).max(b.position.y + b.size.y),
            (self.position.z + self.size.z).max(b.position.z + b.size.z),
        );
        let size = end - position;

        Self { position, size }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_aabb {
        unsafe { std::mem::transmute::<*const Aabb, *const sys::godot_aabb>(self as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_aabb) -> Self {
        unsafe { std::mem::transmute::<sys::godot_aabb, Self>(c) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_types::IsEqualApprox;

    #[test]
    fn test_has_point() {
        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));

        assert!(aabb.has_point(Vector3::new(5.5, 5.5, 5.5)));
        assert!(!aabb.has_point(Vector3::new(1.0, 1.0, 1.0)));
        assert!(!aabb.has_point(Vector3::new(1.0, 1.0, 5.5)));
        assert!(!aabb.has_point(Vector3::new(1.0, 5.5, 1.0)));
        assert!(!aabb.has_point(Vector3::new(5.5, 1.0, 1.0)));
        assert!(!aabb.has_point(Vector3::new(8.0, 8.0, 8.0)));
        assert!(!aabb.has_point(Vector3::new(8.0, 8.0, 5.5)));
        assert!(!aabb.has_point(Vector3::new(8.0, 5.5, 8.0)));
        assert!(!aabb.has_point(Vector3::new(5.5, 8.0, 8.0)));
    }

    #[test]
    fn test_has_point_negative_size() {
        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(-3.0, -3.0, -3.0));

        assert!(!aabb.has_point(Vector3::new(1.5, 1.5, 1.5)));
        assert!(!aabb.has_point(Vector3::new(-1.0, -1.0, -1.0)));
        assert!(!aabb.has_point(Vector3::new(-1.0, -1.0, 1.5)));
        assert!(!aabb.has_point(Vector3::new(-1.0, 1.5, -1.0)));
        assert!(!aabb.has_point(Vector3::new(1.5, -1.0, -1.0)));
        assert!(!aabb.has_point(Vector3::new(4.0, 4.0, 4.0)));
        assert!(!aabb.has_point(Vector3::new(4.0, 4.0, 1.5)));
        assert!(!aabb.has_point(Vector3::new(4.0, 1.5, 4.0)));
        assert!(!aabb.has_point(Vector3::new(1.5, 4.0, 4.0)));

        let aabb = aabb.abs();
        assert!(aabb.has_point(Vector3::new(1.5, 1.5, 1.5)));
        assert!(!aabb.has_point(Vector3::new(-1.0, -1.0, -1.0)));
        assert!(!aabb.has_point(Vector3::new(-1.0, -1.0, 1.5)));
        assert!(!aabb.has_point(Vector3::new(-1.0, 1.5, -1.0)));
        assert!(!aabb.has_point(Vector3::new(1.5, -1.0, -1.0)));
        assert!(!aabb.has_point(Vector3::new(4.0, 4.0, 4.0)));
        assert!(!aabb.has_point(Vector3::new(4.0, 4.0, 1.5)));
        assert!(!aabb.has_point(Vector3::new(4.0, 1.5, 4.0)));
        assert!(!aabb.has_point(Vector3::new(1.5, 4.0, 4.0)));
    }

    #[test]
    fn test_get_axis() {
        let aabb = Aabb::new(Vector3::ZERO, Vector3::new(1.0, 2.0, 3.0));
        assert!(aabb.get_longest_axis().is_equal_approx(Vector3::BACK));
        assert_eq!(aabb.get_longest_axis_index(), Axis::Z);
        assert!(aabb.get_longest_axis_size().is_equal_approx(3.0));

        assert!(aabb.get_shortest_axis().is_equal_approx(Vector3::RIGHT));
        assert_eq!(aabb.get_shortest_axis_index(), Axis::X);
        assert!(aabb.get_shortest_axis_size().is_equal_approx(1.0));
    }

    #[test]
    fn test_grow() {
        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
        let expected = Aabb::new(Vector3::new(1.0, 1.0, 1.0), Vector3::new(7.0, 7.0, 7.0));
        assert_eq!(aabb.grow(2.0), expected);

        let aabb = Aabb::new(Vector3::new(6.0, 6.0, 6.0), Vector3::new(-3.0, -3.0, -3.0));
        let expected = Aabb::new(Vector3::new(4.0, 4.0, 4.0), Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.grow(2.0), expected);
    }

    #[test]
    fn test_intersects() {
        let a = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
        let b = Aabb::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(3.0, 3.0, 3.0));
        let c = Aabb::new(Vector3::new(6.0, 6.0, 6.0), Vector3::new(3.0, 3.0, 3.0));
        let d = Aabb::new(Vector3::new(8.0, 8.0, 8.0), Vector3::new(-3.0, -3.0, -3.0));

        assert!(a.intersects(b));
        assert!(b.intersects(a));

        assert!(!a.intersects(c));
        assert!(!c.intersects(a));
        assert!(!a.intersects(d));
        assert!(!d.intersects(a));

        let d = d.abs();
        assert!(a.intersects(d));
        assert!(d.intersects(a));
    }

    #[test]
    fn test_intersects_plane() {
        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
        let plane_a = Plane::from_points(
            Vector3::ZERO,
            Vector3::new(0.0, 9.0, 9.0),
            Vector3::new(9.0, 9.0, 0.0),
        );
        let plane_b = Plane::from_points(
            Vector3::ZERO,
            Vector3::new(0.0, 9.0, 9.0),
            Vector3::new(0.0, 9.0, -9.0),
        );
        assert!(aabb.intersects_plane(plane_a.unwrap()));
        assert!(!aabb.intersects_plane(plane_b.unwrap()));

        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(-3.0, -3.0, -3.0));
        let plane_a = Plane::from_points(
            Vector3::ZERO,
            Vector3::new(0.0, 3.0, 3.0),
            Vector3::new(3.0, 3.0, 0.0),
        );
        let plane_b = Plane::from_points(
            Vector3::ZERO,
            Vector3::new(0.0, -3.0, 3.0),
            Vector3::new(0.0, -3.0, -3.0),
        );
        assert!(aabb.intersects_plane(plane_a.unwrap()));
        assert!(!aabb.intersects_plane(plane_b.unwrap()));
    }

    #[test]
    fn test_intersects_segment() {
        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
        assert!(aabb.intersects_segment(Vector3::ZERO, Vector3::new(9.0, 9.0, 9.0)));
        assert!(!aabb.intersects_segment(Vector3::ZERO, Vector3::new(9.0, 9.0, 0.0)));

        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(-3.0, -3.0, -3.0));
        assert!(aabb.intersects_segment(Vector3::ZERO, Vector3::new(3.0, 3.0, 3.0)));
        assert!(!aabb.intersects_segment(Vector3::ZERO, Vector3::new(3.0, 3.0, -3.0)));
    }

    #[test]
    fn test_intersection() {
        let a = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
        let b = Aabb::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(3.0, 3.0, 3.0));
        let c = Aabb::new(Vector3::new(6.0, 6.0, 6.0), Vector3::new(3.0, 3.0, 3.0));
        let d = Aabb::new(Vector3::new(8.0, 8.0, 8.0), Vector3::new(-3.0, -3.0, -3.0));

        let expected = Aabb::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(1.0, 1.0, 1.0));

        assert_eq!(a.intersection(b), expected);
        assert_eq!(a.intersection(c), Aabb::default());
        assert_eq!(a.intersection(d), Aabb::default());
        assert_eq!(a.intersection(d.abs()), expected);
    }

    #[test]
    fn test_merge() {
        let a = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(3.0, 3.0, 3.0));
        let b = Aabb::new(Vector3::new(5.0, 5.0, 5.0), Vector3::new(3.0, 3.0, 3.0));
        let c = Aabb::new(Vector3::new(8.0, 8.0, 8.0), Vector3::new(-3.0, -3.0, -3.0));

        let expected = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(5.0, 5.0, 5.0));

        assert_eq!(a.merge(b), expected);
        assert_ne!(a.merge(c), expected);
        assert_eq!(a.merge(c.abs()), expected);
    }
}

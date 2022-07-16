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
    ///
    /// Note that while `size` components are allowed to be negative, they can lead to unintuitive results.
    /// It is recommended to use [`abs`][Self::abs] on such AABBs.
    #[inline]
    pub fn new(position: Vector3, size: Vector3) -> Self {
        Self { position, size }
    }

    /// Ending corner. This is calculated as `position + size`.
    #[inline]
    pub fn end(self) -> Vector3 {
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
    pub fn abs(self) -> Self {
        let position = self.position + Vector3::gd(self.size.glam().min(glam::Vec3A::ZERO));
        let size = self.size.abs();

        Self { position, size }
    }

    /// Returns the volume of the bounding box. See also [`has_no_volume`][Self::has_no_volume].
    ///
    /// This method corresponds to the [`get_area`] GDScript method.
    ///
    /// [`get_area`]: https://docs.godotengine.org/en/stable/classes/class_aabb.html#class-aabb-method-get-area
    #[inline]
    pub fn volume(self) -> f32 {
        self.size.x * self.size.y * self.size.z
    }

    /// Returns true if the bounding box is flat or empty. See also
    /// [`volume`][Self::volume].
    ///
    /// This method corresponds to the [`has_no_area`] GDScript method.
    ///
    /// Note: If the bounding box has a negative size and is not flat or empty, this method will
    /// return true.
    ///
    /// [`has_no_area`]: https://docs.godotengine.org/en/stable/classes/class_aabb.html#class-aabb-method-has-no-area
    #[inline]
    pub fn has_no_volume(self) -> bool {
        self.size.x <= 0.0 || self.size.y <= 0.0 || self.size.z <= 0.0
    }

    /// Returns true if the bounding box is empty or all of its dimensions are negative.
    #[inline]
    pub fn has_no_surface(self) -> bool {
        self.size.x <= 0.0 && self.size.y <= 0.0 && self.size.z <= 0.0
    }

    /// Returns true if the bounding box contains a point. By convention, the right and bottom edges of
    /// the `Rect2` are considered exclusive, so points on these edges are not included.
    ///
    /// Note: This method is not reliable for bounding boxes with a negative size. Use
    /// [`abs`][Self::abs] to get a positive sized equivalent box to check for contained points.
    #[inline]
    pub fn contains_point(self, point: Vector3) -> bool {
        let point = point - self.position;

        point.abs() == point
            && point.x < self.size.x
            && point.y < self.size.y
            && point.z < self.size.z
    }

    /// Returns true if this bounding box and `b` are approximately equal, by calling
    /// [`is_equal_approx`](Vector3::is_equal_approx) on each component.
    #[inline]
    pub fn is_equal_approx(self, b: Self) -> bool {
        self.position.is_equal_approx(b.position) && self.size.is_equal_approx(b.size)
    }

    /// Gets the position of the 8 endpoints of the bounding box in space.
    ///
    /// The index returns an arbitrary point, but all points are guaranteed to be unique.
    #[inline]
    pub fn get_endpoint(self, index: usize) -> Option<Vector3> {
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

    /// Returns the longest side of this AABB as an axis index and its length.
    ///
    /// If multiple axes have the same length, then the first in order X, Y, Z is returned.  
    /// To get the unit vector along the axis, use [`Axis::to_unit_vector()`].
    ///
    /// If you want to emulate the separate GDScript methods, you can do this:
    /// ```no_run
    /// # let aabb: gdnative::core_types::Aabb = todo!();
    /// let (index, size) = aabb.longest_axis();
    /// let axis = index.to_unit_vector();
    /// ```
    #[inline]
    pub fn longest_axis(self) -> (Axis, f32) {
        let Vector3 { x, y, z } = self.size;

        (self.size.max_axis(), x.max(y).max(z))
    }

    /// Returns the shortest side of this AABB as an axis index and its length.
    ///
    /// If multiple axes have the same length, then the first in order X, Y, Z is returned.  
    /// To get the unit vector along the axis, use [`Axis::to_unit_vector()`].
    ///
    /// If you want to emulate the separate GDScript methods, you can do this:
    /// ```no_run
    /// # let aabb: gdnative::core_types::Aabb = todo!();
    /// let (index, size) = aabb.shortest_axis();
    /// let axis = index.to_unit_vector();
    /// ```
    #[inline]
    pub fn shortest_axis(self) -> (Axis, f32) {
        let Vector3 { x, y, z } = self.size;

        (self.size.min_axis(), x.min(y).min(z))
    }

    /// Returns the support point in a given direction. This is useful for collision detection
    /// algorithms.
    ///
    /// The support point is a point on the boundary of the AABB, which is the furthest from the center in the given direction `dir`.
    /// In other words, when you cast a ray from the AABB's center toward `dir` and intersect that with the AABB boundary, you will get
    /// the support point.
    ///
    /// Mathematically, the support point corresponds to the point which maximizes its dot product with `dir`.
    /// See also [1] and [2] for more information.
    ///
    /// [1]: https://ncollide.org/geometric_representations/#support-mappings
    /// [2]: https://www.toptal.com/game/video-game-physics-part-ii-collision-detection-for-solid-objects
    #[inline]
    pub fn get_support(self, dir: Vector3) -> Vector3 {
        self.position
            + Vector3::new(
                if dir.x > 0.0 { 0.0 } else { self.size.x },
                if dir.y > 0.0 { 0.0 } else { self.size.y },
                if dir.z > 0.0 { 0.0 } else { self.size.z },
            )
    }

    /// Returns a copy of the bounding box, grown a given amount of units on all 6 sides.
    ///
    /// It is possible to specify a negative amount to shrink the AABB (note that this can invert the AABB).
    #[inline]
    #[must_use]
    pub fn grow(self, by: f32) -> Self {
        let position = self.position - Vector3::new(by, by, by);
        let size = self.size + Vector3::new(by, by, by) * 2.0;

        Self { position, size }
    }

    /// Returns true if the bounding box overlaps with `b`.
    ///
    /// This **excludes** borders; if the intersection has no volume, `false` is returned.
    #[inline]
    pub fn intersects(self, b: Self) -> bool {
        self.position.x < b.position.x + b.size.x
            && self.position.x + self.size.x > b.position.x
            && self.position.y < b.position.y + b.size.y
            && self.position.y + self.size.y > b.size.y
            && self.position.z < b.position.z + b.size.z
            && self.position.z + self.size.z > b.size.z
    }

    /// Returns true if the bounding box is on both sides of a plane.
    #[inline]
    pub fn intersects_plane(self, plane: Plane) -> bool {
        let mut corners = [Vector3::ZERO; 8];
        for (i, corner) in corners.iter_mut().enumerate() {
            *corner = self.get_endpoint(i).unwrap();
        }

        let mut over = false;
        let mut under = false;
        for point in corners {
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
    pub fn intersects_segment(self, from: Vector3, to: Vector3) -> bool {
        let mut min: f32 = 0.0;
        let mut max: f32 = 1.0;

        for i in 0..3 {
            let from = from.as_ref()[i];
            let to = to.as_ref()[i];
            let begin = self.position.as_ref()[i];
            let end = self.end().as_ref()[i];

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

    /// Returns the intersection between two bounding boxes, or `None` if there is no intersection.
    ///
    /// This **excludes** borders; if the intersection has no volume, `None` is returned.
    #[inline]
    #[must_use]
    pub fn intersection(self, b: Self) -> Option<Self> {
        if !self.intersects(b) {
            return None;
        }

        let mut aabb = b;
        aabb.position.x = aabb.position.x.max(self.position.x);
        aabb.position.y = aabb.position.y.max(self.position.y);
        aabb.position.z = aabb.position.z.max(self.position.z);

        let end = self.end();
        let end_b = b.end();

        aabb.size.x = end.x.min(end_b.x) - aabb.position.x;
        aabb.size.y = end.y.min(end_b.y) - aabb.position.y;
        aabb.size.z = end.y.min(end_b.z) - aabb.position.z;

        Some(aabb)
    }

    /// Returns a larger bounding box that contains both this `Aabb` and `b`.
    #[inline]
    #[must_use]
    pub fn merge(self, b: Self) -> Self {
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

        assert!(aabb.contains_point(Vector3::new(5.5, 5.5, 5.5)));
        assert!(!aabb.contains_point(Vector3::new(1.0, 1.0, 1.0)));
        assert!(!aabb.contains_point(Vector3::new(1.0, 1.0, 5.5)));
        assert!(!aabb.contains_point(Vector3::new(1.0, 5.5, 1.0)));
        assert!(!aabb.contains_point(Vector3::new(5.5, 1.0, 1.0)));
        assert!(!aabb.contains_point(Vector3::new(8.0, 8.0, 8.0)));
        assert!(!aabb.contains_point(Vector3::new(8.0, 8.0, 5.5)));
        assert!(!aabb.contains_point(Vector3::new(8.0, 5.5, 8.0)));
        assert!(!aabb.contains_point(Vector3::new(5.5, 8.0, 8.0)));
    }

    #[test]
    fn test_has_point_negative_size() {
        let aabb = Aabb::new(Vector3::new(3.0, 3.0, 3.0), Vector3::new(-3.0, -3.0, -3.0));

        assert!(!aabb.contains_point(Vector3::new(1.5, 1.5, 1.5)));
        assert!(!aabb.contains_point(Vector3::new(-1.0, -1.0, -1.0)));
        assert!(!aabb.contains_point(Vector3::new(-1.0, -1.0, 1.5)));
        assert!(!aabb.contains_point(Vector3::new(-1.0, 1.5, -1.0)));
        assert!(!aabb.contains_point(Vector3::new(1.5, -1.0, -1.0)));
        assert!(!aabb.contains_point(Vector3::new(4.0, 4.0, 4.0)));
        assert!(!aabb.contains_point(Vector3::new(4.0, 4.0, 1.5)));
        assert!(!aabb.contains_point(Vector3::new(4.0, 1.5, 4.0)));
        assert!(!aabb.contains_point(Vector3::new(1.5, 4.0, 4.0)));

        let aabb = aabb.abs();
        assert!(aabb.contains_point(Vector3::new(1.5, 1.5, 1.5)));
        assert!(!aabb.contains_point(Vector3::new(-1.0, -1.0, -1.0)));
        assert!(!aabb.contains_point(Vector3::new(-1.0, -1.0, 1.5)));
        assert!(!aabb.contains_point(Vector3::new(-1.0, 1.5, -1.0)));
        assert!(!aabb.contains_point(Vector3::new(1.5, -1.0, -1.0)));
        assert!(!aabb.contains_point(Vector3::new(4.0, 4.0, 4.0)));
        assert!(!aabb.contains_point(Vector3::new(4.0, 4.0, 1.5)));
        assert!(!aabb.contains_point(Vector3::new(4.0, 1.5, 4.0)));
        assert!(!aabb.contains_point(Vector3::new(1.5, 4.0, 4.0)));
    }

    #[test]
    fn test_longest_shortest_axis() {
        let aabb = Aabb::new(Vector3::ZERO, Vector3::new(1.0, 2.0, 3.0));

        let (longest_axis, longest_size) = aabb.longest_axis();
        let longest_vector = longest_axis.to_unit_vector();

        assert!(longest_vector.is_equal_approx(Vector3::BACK));
        assert_eq!(longest_axis, Axis::Z);
        assert!(longest_size.is_equal_approx(3.0));

        let (shortest_axis, shortest_size) = aabb.shortest_axis();
        let shortest_vector = shortest_axis.to_unit_vector();

        assert!(shortest_vector.is_equal_approx(Vector3::RIGHT));
        assert_eq!(shortest_axis, Axis::X);
        assert!(shortest_size.is_equal_approx(1.0));
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

        assert!(a.intersects(a));
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

        assert_eq!(a.intersection(a), Some(a));
        assert_eq!(a.intersection(b), Some(expected));
        assert_eq!(a.intersection(c), None);
        assert_eq!(a.intersection(d), None);
        assert_eq!(a.intersection(d.abs()), Some(expected));
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

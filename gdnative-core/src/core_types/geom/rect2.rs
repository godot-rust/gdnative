use crate::core_types::Vector2;
use std::convert::TryFrom;

/// 2D axis-aligned bounding box.
///
/// `Rect2` consists of a position, a size, and several utility functions. It is typically used for
/// fast overlap tests.
///
/// The 3D counterpart to `Rect2` is [`Aabb`](crate::core_types::Aabb).
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Rect2 {
    /// The rectangle's position in 2D space.
    pub position: Vector2,
    /// Width and height.
    pub size: Vector2,
}

impl Rect2 {
    /// Creates a `Rect2` by position and size.
    #[inline]
    pub fn new(position: Vector2, size: Vector2) -> Self {
        Self { position, size }
    }

    /// Creates a `Rect2` by x, y, width, and height.
    #[inline]
    pub fn from_components(x: f32, y: f32, width: f32, height: f32) -> Self {
        let position = Vector2::new(x, y);
        let size = Vector2::new(width, height);

        Self { position, size }
    }

    /// Ending corner. This is calculated as `position + size`.
    #[inline]
    pub fn end(self) -> Vector2 {
        self.position + self.size
    }

    /// Ending corner. Setting this value will change the size.
    #[inline]
    pub fn set_end(&mut self, new_end: Vector2) {
        self.size = new_end - self.position;
    }

    /// Returns a rectangle with equivalent position and area, modified so that the top-left corner
    /// is the origin and `width` and `height` are positive.
    #[inline]
    pub fn abs(self) -> Self {
        let position = self.position + Vector2::new(self.size.x.min(0.0), self.size.y.min(0.0));
        let size = self.size.abs();

        Self { position, size }
    }

    /// Returns the area of the rectangle. See also [`has_no_area`][Self::has_no_area].
    #[inline]
    pub fn area(self) -> f32 {
        self.size.x * self.size.y
    }

    /// Returns true if the rectangle is flat or empty. See also [`area`][Self::area].
    ///
    /// Note: If the `Rect2` has a negative size and is not flat or empty, this method will return
    /// true. Use [`abs`][Self::abs] to make the size positive.
    ///
    /// # Example
    ///
    /// ```
    /// # use gdnative::prelude::*;
    /// # fn main() {
    /// let rect = Rect2::new(
    ///     Vector2::new(2.0, 3.0),
    ///     Vector2::new(-3.0, -4.0),
    /// );
    /// assert!(rect.has_no_area());
    /// assert!(!rect.abs().has_no_area());
    /// # }
    /// ```
    #[inline]
    pub fn has_no_area(self) -> bool {
        self.size.x <= 0.0 || self.size.y <= 0.0
    }

    /// Returns true if the rectangle contains a point. By convention, the right and bottom edges of
    /// the rectangle are considered exclusive, so points on these edges are not included.
    ///
    /// Note: This method is not reliable for `Rect2` with a negative size. Use [`abs`][Self::abs]
    /// to get a positive sized equivalent rectangle to check for contained points.
    #[inline]
    pub fn contains_point(self, point: Vector2) -> bool {
        let point = point - self.position;

        point.abs() == point && point.x < self.size.x && point.y < self.size.y
    }

    /// Returns true if this rectangle and `b` are approximately equal, by calling
    /// [`is_equal_approx`](Vector2::is_equal_approx) on each component.
    #[inline]
    pub fn is_equal_approx(self, b: Self) -> bool {
        self.position.is_equal_approx(b.position) && self.size.is_equal_approx(b.size)
    }

    /// Returns true if the inside of the rectangle overlaps with `b` (i.e. they have at least one point in
    /// common).
    ///
    /// This **excludes** borders. See [`intersects_including_borders`][Self::intersects_including_borders] for inclusive check.
    ///
    /// Note: This method is not reliable for `Rect2` with a negative size. Use [`abs`][Self::abs]
    /// to get a positive sized equivalent rectangle to check for intersections.
    #[inline]
    pub fn intersects(self, b: Self) -> bool {
        self.position.x < b.position.x + b.size.x
            && self.position.x + self.size.x > b.position.x
            && self.position.y < b.position.y + b.size.y
            && self.position.y + self.size.y > b.position.y
    }

    /// Returns true if the rectangle overlaps with `b` (i.e. they have at least one point in
    /// common) or their borders touch even without intersection.
    ///
    /// This **includes** borders. See [`intersects`][Self::intersects] for exclusive check.
    ///
    /// Note: This method is not reliable for `Rect2` with a negative size. Use [`abs`][Self::abs]
    /// to get a positive sized equivalent rectangle to check for intersections.
    #[inline]
    pub fn intersects_including_borders(self, b: Self) -> bool {
        self.position.x <= b.position.x + b.size.x
            && self.position.x + self.size.x >= b.position.x
            && self.position.y <= b.position.y + b.size.y
            && self.position.y + self.size.y >= b.position.y
    }

    /// Returns true if this rectangle (inclusively) encloses `b`.
    ///
    /// This is true when `self` covers all the area of `b`, and possibly (but not necessarily) more.
    #[inline]
    pub fn encloses(self, b: Self) -> bool {
        b.position.x >= self.position.x
            && b.position.y >= self.position.y
            && b.position.x + b.size.x <= self.position.x + self.size.x
            && b.position.y + b.size.y <= self.position.y + self.size.y
    }

    /// Returns the intersection of this rectangle and `b`, or `None` if they don't intersect.
    ///
    /// This is similar to the GDScript `clip` function, but returns `None` instead of `self` if there is no intersection.
    /// This method **excludes** borders just like [`intersects`][Self::intersects].
    ///
    /// Note: This method is not reliable for `Rect2` with a negative size. Use [`abs`][Self::abs]
    /// to get a positive sized equivalent rectangle for clipping.
    #[inline]
    #[must_use]
    pub fn intersection(self, b: Self) -> Option<Self> {
        if !self.intersects(b) {
            return None;
        }

        let mut rect = b;
        rect.position.x = rect.position.x.max(self.position.x);
        rect.position.y = rect.position.y.max(self.position.y);

        let end = self.end();
        let end_b = b.end();

        rect.size.x = end.x.min(end_b.x) - rect.position.x;
        rect.size.y = end.y.min(end_b.y) - rect.position.y;

        Some(rect)
    }

    /// Returns a larger rectangle that contains this `Rect2` and `b`.
    ///
    /// Note: This method is not reliable for `Rect2` with a negative size. Use [`abs`][Self::abs]
    /// to get a positive sized equivalent rectangle for merging.
    #[inline]
    #[must_use]
    pub fn merge(self, b: Self) -> Self {
        let position = Vector2::new(
            self.position.x.min(b.position.x),
            self.position.y.min(b.position.y),
        );
        let end = Vector2::new(
            (self.position.x + self.size.x).max(b.position.x + b.size.x),
            (self.position.y + self.size.y).max(b.position.y + b.size.y),
        );
        let size = end - position;

        Self { position, size }
    }

    /// Returns a copy of this rectangle expanded to include a given point.
    ///
    /// Note: This method is not reliable for `Rect2` with a negative size. Use [`abs`][Self::abs]
    /// to get a positive sized equivalent rectangle for expanding.
    ///
    /// # Example
    ///
    /// ```
    /// # use gdnative::prelude::*;
    /// # fn main() {
    /// let rect = Rect2::new(
    ///     Vector2::new(-3.0, 2.0),
    ///     Vector2::new(1.0, 1.0),
    /// );
    ///
    /// let rect2 = rect.expand(Vector2::new(0.0, -1.0));
    ///
    /// assert_eq!(rect2.position, Vector2::new(-3.0, -1.0));
    /// assert_eq!(rect2.size, Vector2::new(3.0, 4.0));
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn expand(self, to: Vector2) -> Self {
        self.merge(Self::new(to, Vector2::ZERO))
    }

    /// Returns a copy of this rectangle grown by a given amount of units on all the sides.
    #[inline]
    #[must_use]
    pub fn grow(self, by: f32) -> Self {
        let position = self.position - Vector2::new(by, by);
        let size = self.size + Vector2::new(by, by) * 2.0;

        Self { position, size }
    }

    /// Returns a copy of this rectangle grown by a given amount of units towards each direction
    /// individually.
    #[inline]
    #[must_use]
    pub fn grow_individual(mut self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        self.position.x -= left;
        self.position.y -= top;
        self.size.x += left + right;
        self.size.y += top + bottom;

        self
    }

    /// Returns a copy of this rectangle grown by a given amount of units towards the [`Margin`]
    /// direction.
    #[inline]
    #[must_use]
    pub fn grow_margin(self, margin: Margin, amount: f32) -> Self {
        let left = if margin == Margin::Left { amount } else { 0.0 };
        let top = if margin == Margin::Top { amount } else { 0.0 };
        let right = if margin == Margin::Right { amount } else { 0.0 };
        let bottom = if margin == Margin::Bottom {
            amount
        } else {
            0.0
        };

        self.grow_individual(left, top, right, bottom)
    }
}

/// Error indicating that an `i64` cannot be converted to a [`Margin`].
#[derive(Debug)]
pub struct MarginError(i64);

/// Provides compatibility with Godot's [`Margin` enum][margin] through the [`TryFrom`] trait.
///
/// [margin]: https://docs.godotengine.org/en/stable/classes/class_%40globalscope.html#enum-globalscope-margin
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Margin {
    Left,
    Top,
    Right,
    Bottom,
}

impl TryFrom<i64> for Margin {
    type Error = MarginError;

    #[inline]
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        // XXX: Can't use the constants defined in `gdnative_bindings::GlobalConstants`
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Top),
            2 => Ok(Self::Right),
            3 => Ok(Self::Bottom),
            _ => Err(MarginError(value)),
        }
    }
}

impl From<Margin> for i64 {
    #[inline]
    fn from(margin: Margin) -> Self {
        // XXX: Can't use the constants defined in `gdnative_bindings::GlobalConstants`
        match margin {
            Margin::Left => 0,
            Margin::Top => 1,
            Margin::Right => 2,
            Margin::Bottom => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_point() {
        let rect = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));

        assert!(rect.contains_point(Vector2::new(5.5, 5.5)));
        assert!(!rect.contains_point(Vector2::new(1.0, 1.0)));
        assert!(!rect.contains_point(Vector2::new(1.0, 5.5)));
        assert!(!rect.contains_point(Vector2::new(5.5, 1.0)));
        assert!(!rect.contains_point(Vector2::new(8.0, 8.0)));
        assert!(!rect.contains_point(Vector2::new(8.0, 5.5)));
        assert!(!rect.contains_point(Vector2::new(5.5, 8.0)));
    }

    #[test]
    fn test_contains_point_negative_size() {
        let rect = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(-3.0, -3.0));

        assert!(!rect.contains_point(Vector2::new(1.5, 1.5)));
        assert!(!rect.contains_point(Vector2::new(-1.0, -1.0)));
        assert!(!rect.contains_point(Vector2::new(-1.0, 1.5)));
        assert!(!rect.contains_point(Vector2::new(1.5, -1.0)));
        assert!(!rect.contains_point(Vector2::new(4.0, 4.0)));
        assert!(!rect.contains_point(Vector2::new(4.0, 1.5)));
        assert!(!rect.contains_point(Vector2::new(1.5, 4.0)));

        let rect = rect.abs();
        assert!(rect.contains_point(Vector2::new(1.5, 1.5)));
        assert!(!rect.contains_point(Vector2::new(-1.0, -1.0)));
        assert!(!rect.contains_point(Vector2::new(-1.0, 1.5)));
        assert!(!rect.contains_point(Vector2::new(1.5, -1.0)));
        assert!(!rect.contains_point(Vector2::new(4.0, 4.0)));
        assert!(!rect.contains_point(Vector2::new(4.0, 1.5)));
        assert!(!rect.contains_point(Vector2::new(1.5, 4.0)));
    }

    #[test]
    fn test_intersects() {
        let a = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let b = Rect2::new(Vector2::new(5.0, 5.0), Vector2::new(3.0, 3.0));
        let c = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(3.0, 3.0));
        let d = Rect2::new(Vector2::new(8.0, 8.0), Vector2::new(-3.0, -3.0));

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
    fn test_intersects_including_borders() {
        let a = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let b = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(3.0, 3.0));
        let c = Rect2::new(Vector2::new(9.0, 9.0), Vector2::new(-3.0, -3.0));

        assert!(a.intersects_including_borders(a));

        assert!(a.intersects_including_borders(b));
        assert!(b.intersects_including_borders(a));

        assert!(!a.intersects_including_borders(c));
        assert!(!c.intersects_including_borders(a));

        let c = c.abs();
        assert!(a.intersects_including_borders(c));
        assert!(c.intersects_including_borders(a));
    }

    #[test]
    fn test_encloses() {
        let a = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let b = Rect2::new(Vector2::new(4.0, 4.0), Vector2::new(2.0, 2.0));
        let c = Rect2::new(Vector2::new(5.0, 5.0), Vector2::new(2.0, 2.0));
        let d = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-2.0, -2.0));

        assert!(a.encloses(a));
        assert!(a.encloses(b));
        assert!(!a.encloses(c));
        assert!(a.encloses(d));
    }

    #[test]
    fn test_intersection() {
        let a = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let b = Rect2::new(Vector2::new(5.0, 5.0), Vector2::new(3.0, 3.0));
        let c = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(3.0, 3.0));
        let d = Rect2::new(Vector2::new(8.0, 8.0), Vector2::new(-3.0, -3.0));

        let expected = Rect2::new(Vector2::new(5.0, 5.0), Vector2::new(1.0, 1.0));

        assert_eq!(a.intersection(a), Some(a));
        assert_eq!(a.intersection(b), Some(expected));
        assert_eq!(a.intersection(c), None);
        assert_eq!(a.intersection(d), None);
        assert_eq!(a.intersection(d.abs()), Some(expected));
    }

    #[test]
    fn test_merge() {
        let a = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let b = Rect2::new(Vector2::new(5.0, 5.0), Vector2::new(3.0, 3.0));
        let c = Rect2::new(Vector2::new(8.0, 8.0), Vector2::new(-3.0, -3.0));

        let expected = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(5.0, 5.0));

        assert_eq!(a.merge(b), expected);
        assert_ne!(a.merge(c), expected);
        assert_eq!(a.merge(c.abs()), expected);
    }

    #[test]
    fn test_expand() {
        let a = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let b = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-3.0, -3.0));

        let begin = Vector2::new(-1.0, -1.0);
        let expected = Rect2::new(begin, Vector2::new(7.0, 7.0));

        assert_eq!(a.expand(begin), expected);
        assert_ne!(b.expand(begin), expected);
        assert_eq!(b.abs().expand(begin), expected);
    }

    #[test]
    fn test_grow() {
        let rect = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let expected = Rect2::new(Vector2::new(1.0, 1.0), Vector2::new(7.0, 7.0));
        assert_eq!(rect.grow(2.0), expected);

        let rect = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-3.0, -3.0));
        let expected = Rect2::new(Vector2::new(4.0, 4.0), Vector2::new(1.0, 1.0));
        assert_eq!(rect.grow(2.0), expected);
    }

    #[test]
    fn test_grow_individual() {
        let rect = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        let expected = Rect2::new(Vector2::new(2.0, 1.0), Vector2::new(7.0, 9.0));
        assert_eq!(rect.grow_individual(1.0, 2.0, 3.0, 4.0), expected);

        let rect = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-3.0, -3.0));
        let expected = Rect2::new(Vector2::new(5.0, 4.0), Vector2::new(1.0, 3.0));
        assert_eq!(rect.grow_individual(1.0, 2.0, 3.0, 4.0), expected);
    }

    #[test]
    fn test_grow_margin() {
        let rect = Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 3.0));
        assert_eq!(
            rect.grow_margin(Margin::Left, 2.0),
            Rect2::new(Vector2::new(1.0, 3.0), Vector2::new(5.0, 3.0)),
        );
        assert_eq!(
            rect.grow_margin(Margin::Right, 2.0),
            Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(5.0, 3.0)),
        );
        assert_eq!(
            rect.grow_margin(Margin::Top, 2.0),
            Rect2::new(Vector2::new(3.0, 1.0), Vector2::new(3.0, 5.0)),
        );
        assert_eq!(
            rect.grow_margin(Margin::Bottom, 2.0),
            Rect2::new(Vector2::new(3.0, 3.0), Vector2::new(3.0, 5.0)),
        );

        let rect = Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-3.0, -3.0));
        assert_eq!(
            rect.grow_margin(Margin::Left, 2.0),
            Rect2::new(Vector2::new(4.0, 6.0), Vector2::new(-1.0, -3.0)),
        );
        assert_eq!(
            rect.grow_margin(Margin::Right, 2.0),
            Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-1.0, -3.0)),
        );
        assert_eq!(
            rect.grow_margin(Margin::Top, 2.0),
            Rect2::new(Vector2::new(6.0, 4.0), Vector2::new(-3.0, -1.0)),
        );
        assert_eq!(
            rect.grow_margin(Margin::Bottom, 2.0),
            Rect2::new(Vector2::new(6.0, 6.0), Vector2::new(-3.0, -1.0)),
        );
    }
}

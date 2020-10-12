use crate::core_types::Vector3;
use euclid::approxeq::ApproxEq;

/// Plane in hessian form.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

impl Plane {
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_plane {
        unsafe { std::mem::transmute::<*const Plane, *const sys::godot_plane>(self as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_plane) -> Self {
        unsafe { std::mem::transmute::<sys::godot_plane, Self>(c) }
    }

    /// Creates a new `Plane` from the ['Vector3'](./type.Vector3.html) normal and the distance from the origin.
    #[inline]
    pub fn new_with_normal(normal: Vector3, d: f32) -> Plane {
        Plane {
            normal: normal,
            d: d,
        }
    }

    /// Creates a new `Plane` from four floats.
    /// a, b, c are used for the normal ['Vector3'](./type.Vector3.html).
    /// d is the distance from the origin.
    #[inline]
    pub fn new_with_reals(a: f32, b: f32, c: f32, d: f32) -> Plane {
        Plane {
            normal: Vector3::new(a, b, c),
            d: d,
        }
    }

    /// Creates a new `Plane` from three [`Vector3`](./type.Vector3.html), given in clockwise order.
    #[inline]
    pub fn new_with_vectors(a: Vector3, b: Vector3, c: Vector3) -> Plane {
        let normal = (a - c).cross(a - b);

        Plane {
            normal: normal,
            d: normal.dot(a),
        }
    }

    /// Returns the center of the `Plane`.
    #[inline]
    pub fn center(&self) -> Vector3 {
        self.normal * self.d
    }

    /// Returns the shortest distance from the `Plane` to `point`.
    #[inline]
    pub fn distance_to(&self, point: Vector3) -> f32 {
        (self.normal.dot(point)) - self.d
    }

    /// Returns `true` if `point` is inside the `Plane`.
    /// `epislon` specifies the minimum threshold to be considered inside the `Plane`.
    #[inline]
    pub fn has_point(&self, point: Vector3, epsilon: f32) -> bool {
        let dist = self.distance_to(point).abs();

        dist <= epsilon
    }

    /// Returns the intersection point of the three planes `b`, `c` and this `Plane`.
    /// Returns `None` if the 'Plane's don't intersect.
    #[inline]
    pub fn intersect_3(&self, b: Plane, c: Plane) -> Option<Vector3> {
        let a = &self;

        let denom = Vector3::cross(a.normal, b.normal).dot(c.normal);

        if denom.approx_eq(&0.0) {
            None
        } else {
            Some(
                ((Vector3::cross(b.normal, c.normal) * a.d)
                    + (Vector3::cross(c.normal, a.normal) * b.d)
                    + (Vector3::cross(a.normal, b.normal) * c.d))
                    / denom,
            )
        }
    }

    /// Returns the intersection point of a ray consisting of the position `from` and the direction normal `dir` with this plane/
    /// Returns `None` if the ray doesn't intersect.
    #[inline]
    pub fn intersects_ray(&self, from: Vector3, dir: Vector3) -> Option<Vector3> {
        let den = self.normal.dot(dir);

        if den.approx_eq(&0.0) {
            return None;
        }

        let dist = (self.normal.dot(from) - self.d) / den;

        if dist > std::f32::EPSILON {
            return None;
        }

        Some(from + dir * -dist)
    }

    /// Returns the intersection point of a segment from `begin` to `end` with this `Plane`.
    /// Returns `None` if the the segment doesn't intersect.
    #[inline]
    pub fn intersects_segment(&self, begin: Vector3, end: Vector3) -> Option<Vector3> {
        let segment = begin - end;
        let den = self.normal.dot(segment);

        if den.approx_eq(&0.0) {
            return None;
        }

        let dist = (self.normal.dot(begin) - self.d) / den;

        if dist < -std::f32::EPSILON || dist > (1.0 + std::f32::EPSILON) {
            return None;
        }

        Some(begin + segment * -dist)
    }

    /// Returns `true` if this `Plane` and `other` are approximately equal.
    /// Determined by running `approx_eq` on both `normal` and `d`.
    #[inline]
    pub fn approx_eq(&self, other: Plane) -> bool {
        self.normal.approx_eq(&other.normal) && self.d.approx_eq(&other.d)
    }

    /// Returns `true` if `point` is above the `Plane`.
    #[inline]
    pub fn is_point_over(&self, point: Vector3) -> bool {
        self.normal.dot(point) > self.d
    }

    /// Returns the `Plane` normalized.
    #[inline]
    pub fn normalize(mut self) -> Plane {
        let l = self.normal.length();

        if l == 0.0 {
            self.normal = Vector3::new(0.0, 0.0, 0.0);
            self.d = 0.0;
            return self;
        } else {
            self.normal /= l;
            self.d /= l;
            return self;
        }
    }

    /// Returns the orthogonal projection of `point` into a point in the `Plane`.
    #[inline]
    pub fn project(&self, point: Vector3) -> Vector3 {
        point - self.normal * self.distance_to(point)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_inputs() -> (Plane, Vector3) {
        (
            Plane::new_with_reals(0.01, 0.02, 0.04, 0.08),
            Vector3::new(0.16, 0.32, 0.64),
        )
    }

    #[test]
    fn center() {
        let (p, _v) = test_inputs();

        let expected = Vector3::new(0.0008, 0.0016, 0.0032);

        assert_eq!(p.center(), expected);
    }

    #[test]
    fn distance_to() {
        let (p, v) = test_inputs();

        let expected = -0.0464;

        assert_eq!(p.distance_to(v), expected);
    }

    #[test]
    fn has_point() {
        let p = Plane::new_with_normal(Vector3::new(1.0, 1.0, 1.0), 1.0);

        let outside = Vector3::new(0.0, 0.0, 0.0);
        let inside = Vector3::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);

        assert!(!p.has_point(outside, 0.00001));
        assert!(p.has_point(inside, 0.00001));
    }

    #[test]
    fn intersect_3() {
        let (p, _v) = test_inputs();

        let b = Plane::new_with_reals(0.08, 0.04, 0.03, 0.01);
        let c = Plane::new_with_reals(0.05, 0.2, 0.1, 0.6);

        let expected = Vector3::new(-1.707317, 2.95122, 0.95122);

        let d = Plane::new_with_reals(0.01, 0.02, 0.4, 0.16);
        let e = Plane::new_with_reals(0.01, 0.02, 0.4, 0.32);

        assert!(p.intersect_3(b, c).unwrap().approx_eq(&expected));
        assert_eq!(p.intersect_3(d, e), None);
    }

    #[test]
    fn intersects_ray() {
        let (p, v) = test_inputs();

        let expected = Vector3::new(0.16, 2.64, 0.64);

        assert!(p
            .intersects_ray(v, Vector3::new(0.0, 1.0, 0.0))
            .unwrap()
            .approx_eq(&expected));
        assert_eq!(p.intersects_ray(v, Vector3::new(0.0, -1.0, 0.0)), None);
    }

    #[test]
    fn intersects_segment() {
        let (p, v) = test_inputs();

        let expected = Vector3::new(0.16, 2.64, 0.64);

        assert!(p
            .intersects_segment(v, Vector3::new(0.16, 10.0, 0.64))
            .unwrap()
            .approx_eq(&expected));
        assert_eq!(
            p.intersects_segment(v, Vector3::new(0.16, -10.0, 0.64)),
            None
        );
    }

    #[test]
    fn is_point_over() {
        let (p, v) = test_inputs();

        assert!(!p.is_point_over(v));
        assert!(p.is_point_over(Vector3::new(1.0, 10.0, 2.0)));
    }

    #[test]
    fn normalize() {
        let (p, _v) = test_inputs();

        assert!(p.normalize().approx_eq(Plane::new_with_reals(
            0.218218, 0.436436, 0.872872, 1.745743
        )));
    }

    #[test]
    fn project() {
        let (p, v) = test_inputs();

        let expected = Vector3::new(0.160464, 0.320928, 0.641856);

        assert!(p.project(v).approx_eq(&expected))
    }
}

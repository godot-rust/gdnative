use crate::core_types::{IsEqualApprox, Vector3};

// TODO(#994) enforce invariants via setters, make fields private
// Otherwise almost all methods need to panic
// - normal.length() == 1
// - d > 0

/// 3D plane in Hessian form: `a*b + b*y + c*z + d = 0`
///
/// Note: almost all methods on `Plane` require that the `normal` vector have
/// unit length and will panic if this invariant is violated. This is not separately
/// annotated for each method.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plane {
    /// Normal vector, perpendicular to the plane.
    ///
    /// Most `Plane` methods expect this vector to be normalized and may panic if this is not the case.
    pub normal: Vector3,

    /// Distance from the coordinate system origin (in the direction of `normal`).
    ///
    /// This value is typically non-negative. It can however be negative, which behaves as if `normal` changed direction.
    pub d: f32,
}

impl Plane {
    /// Creates a new `Plane` from the `normal` and the distance from the origin `d`.
    ///
    /// # Panics
    /// In contrast to construction via `Plane { normal, d }`, this verifies that `normal` has unit length, and will
    /// panic if this is not the case.
    #[inline]
    pub fn new(normal: Vector3, d: f32) -> Self {
        // Design: we could call normalize() here, however that suggests to the user that vectors with non-unit
        // length are valid normals, and tempts users to assign those directly to the field. It's also confusing
        // if Plane { normal, d } and Plane::new(normal, d) have fundamentally different behaviors.
        // If invariants were enforced at the field level using setters, the story might be different.

        let result = Self { normal, d };
        result.ensure_normalized();
        result
    }

    /// Creates a new `Plane` from normal and origin distance.
    ///
    /// `a`, `b`, `c` are used for the `normal` vector.
    /// `d` is the distance from the origin.
    ///
    /// # Panics
    /// See [`Self::new()`].
    #[inline]
    pub fn from_coordinates(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self::new(Vector3::new(a, b, c), d)
    }

    /// Creates a new `Plane` from three [`Vector3`](./type.Vector3.html), given in clockwise order.
    ///
    /// If all three points are collinear, returns `None`.
    #[inline]
    pub fn from_points(a: Vector3, b: Vector3, c: Vector3) -> Option<Self> {
        let normal = (a - c).cross(a - b).normalized();

        if normal.x.is_nan() || normal.y.is_nan() || normal.z.is_nan() {
            None
        } else {
            Some(Self {
                normal,
                d: normal.dot(a),
            })
        }
    }

    /// Returns the point on the `Plane`, which is closest to the origin.
    ///
    /// This is equivalent to `self.project(Vector3::ZERO)`.
    #[inline]
    pub fn center(self) -> Vector3 {
        self.ensure_normalized();

        self.normal * self.d
    }

    /// Returns the orthogonal projection of `point` onto a point in the `Plane`.
    ///
    /// The projection is a point, which lies on the plane and is closest to `point`.
    #[inline]
    pub fn project(self, point: Vector3) -> Vector3 {
        // Note: invariant check in distance_to()

        point - self.normal * self.distance_to(point)
    }

    /// Returns the **signed** distance from the `Plane` to `point`.
    ///
    /// This value is negative, if `self.is_point_over(point)` is false.
    #[inline]
    pub fn distance_to(self, point: Vector3) -> f32 {
        self.ensure_normalized();

        (self.normal.dot(point)) - self.d
    }

    /// Returns `true` if `point` is inside the `Plane`.
    ///
    /// Uses a default epsilon for the boundary check. Use [`Self::contains_point_eps()`] for more control.
    #[inline]
    pub fn contains_point(self, point: Vector3) -> bool {
        // Note: invariant check in distance_to()

        self.contains_point_eps(point, crate::core_types::CMP_EPSILON as f32)
    }

    /// Returns `true` if `point` is inside the `Plane`.
    ///
    /// `epsilon` specifies the minimum distance, at and below which a point is considered inside the `Plane`.
    #[inline]
    pub fn contains_point_eps(self, point: Vector3, epsilon: f32) -> bool {
        // Note: invariant check in distance_to()

        let dist = self.distance_to(point).abs();
        dist <= epsilon
    }

    /// Returns the intersection point of the three planes `self`, `b` and `c`.
    ///
    /// Returns `None` if the planes don't intersect.
    #[inline]
    pub fn intersect_3(self, b: Plane, c: Plane) -> Option<Vector3> {
        self.ensure_normalized();
        b.ensure_normalized();
        c.ensure_normalized();

        let a = self;
        let denom = Vector3::cross(a.normal, b.normal).dot(c.normal);

        if denom.is_equal_approx(0.0) {
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

    /// Returns the intersection point of a ray consisting of the position `from` and the direction vector `dir` with this plane.
    ///
    /// Returns `None` if the ray doesn't intersect.
    #[inline]
    pub fn intersect_ray(self, from: Vector3, dir: Vector3) -> Option<Vector3> {
        self.ensure_normalized();

        let denom = self.normal.dot(dir);
        if denom.is_equal_approx(0.0) {
            return None;
        }

        let dist = (self.normal.dot(from) - self.d) / denom;
        if dist > f32::EPSILON {
            return None;
        }

        Some(from + dir * -dist)
    }

    /// Returns the intersection point of a segment from `begin` to `end` with this `Plane`.
    ///
    /// Returns `None` if the the segment doesn't intersect.
    #[inline]
    pub fn intersect_segment(self, begin: Vector3, end: Vector3) -> Option<Vector3> {
        self.ensure_normalized();

        let segment = begin - end;
        let denom = self.normal.dot(segment);

        if denom.is_equal_approx(0.0) {
            return None;
        }

        let dist = (self.normal.dot(begin) - self.d) / denom;

        // check that dist is not in -EPSILON..(EPSILON+1)
        if (-f32::EPSILON..=(f32::EPSILON + 1.0)).contains(&dist) {
            Some(begin + segment * -dist)
        } else {
            None
        }
    }

    /// Returns `true` if this `Plane` and `other` are approximately equal.
    #[inline]
    pub fn is_equal_approx(self, other: Plane) -> bool {
        self.normal.is_equal_approx(other.normal) && self.d.is_equal_approx(other.d)
    }

    /// Returns `true` if `point` is on the side of the `Plane`, into which `normal` points.
    ///
    /// Points that lie exactly on the plane will be returned as `false`.
    #[inline]
    pub fn is_point_over(self, point: Vector3) -> bool {
        self.ensure_normalized();

        self.normal.dot(point) > self.d
    }

    /// Returns the `Plane` normalized.
    ///
    /// # Panics
    /// If `self.normal` is a zero vector. All other vectors are explicitly allowed.
    #[inline]
    pub fn normalized(mut self) -> Self {
        let l = self.normal.length();

        assert_ne!(
            l, 0.0,
            "Plane::normal {:?} must not be a zero vector",
            self.normal
        );

        self.normal /= l;
        self.d /= l;
        self
    }

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

    #[inline]
    fn ensure_normalized(self) {
        assert!(
            self.normal.is_normalized(),
            "Plane {:?} -- normal does not have unit length",
            self.normal
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_inputs() -> (Plane, Vector3, Vector3) {
        (
            Plane::new(Vector3::new(2.0, 3.0, 7.0).normalized(), 1.5),
            Vector3::new(8.0, 2.0, 5.0),
            Vector3::new(-1.0, 5.0, -3.0),
        )
    }

    #[test]
    fn from_points() {
        let a = Vector3::new(-1.0, 1.0, 0.0);
        let b = Vector3::new(-1.0, 0.0, 0.0);
        let c = Vector3::new(1.0, 1.0, 1.0);
        let d = Vector3::new(-1.0, -1.0, 0.0);

        let expected_valid = Plane::from_coordinates(0.447214, 0.0, -0.894427, -0.447214);

        assert!(Plane::from_points(a, b, c)
            .unwrap()
            .is_equal_approx(expected_valid));

        assert_eq!(Plane::from_points(a, b, d), None);
    }

    #[test]
    fn center() {
        let (p, ..) = test_inputs();
        let expected = Vector3::new(0.381, 0.571501, 1.333501);

        assert!(p.center().is_equal_approx(expected));
    }

    #[test]
    fn project() {
        let (p, v, w) = test_inputs();
        let expected_o = Vector3::new(0.381, 0.571501, 1.333501);
        let expected_v = Vector3::new(6.542291, -0.186564, -0.101982);
        let expected_w = Vector3::new(-0.360935, 5.958597, -0.763273);

        assert!(p.project(Vector3::ZERO).is_equal_approx(expected_o));
        assert!(p.project(v).is_equal_approx(expected_v));
        assert!(p.project(w).is_equal_approx(expected_w));
    }

    #[test]
    fn distance_to() {
        let (p, v, w) = test_inputs();

        let expected_o = -p.d; // negative, because the origin is on opposite side of plane's normal
        let expected_v = 5.739007;
        let expected_w = -2.516001;

        assert!(p.distance_to(Vector3::ZERO).is_equal_approx(expected_o));
        assert!(p.distance_to(v).is_equal_approx(expected_v));
        assert!(p.distance_to(w).is_equal_approx(expected_w));
    }

    #[test]
    fn contains_point() {
        let (p, ..) = test_inputs();

        // points from project()
        assert!(p.contains_point(Vector3::new(0.381, 0.571501, 1.333501)));
        assert!(p.contains_point(Vector3::new(6.542291, -0.186564, -0.101982)));

        // slightly modified X coord
        assert!(!p.contains_point(Vector3::new(6.552291, -0.186564, -0.101982)));
        assert!(!p.contains_point(Vector3::new(6.562291, -0.186564, -0.101982)));
    }

    // TODO(#994) contains_point_eps()

    #[test]
    fn intersect_3() {
        let (p, ..) = test_inputs();

        let p2 = Plane::new(Vector3::new(3.0, 1.0, -2.0).normalized(), 1.0);
        let p3 = Plane::new(Vector3::new(7.0, -4.0, 5.0).normalized(), 0.0);

        let q = Plane::new(p.normal, 2.5); // parallel to p

        let expected = Vector3::new(0.868692, 2.161257, 0.512837);

        assert!(p.intersect_3(p2, p3).unwrap().is_equal_approx(expected));
        assert!(p.intersect_3(p3, p2).unwrap().is_equal_approx(expected));

        assert_eq!(p.intersect_3(q, p2), None);
        assert_eq!(p.intersect_3(q, p3), None);
    }

    #[test]
    fn intersects_ray() {
        let (p, v, w) = test_inputs();

        let expected = Vector3::new(1.743063, 4.085646, -0.561722);

        assert!(p.intersect_ray(v, w - v).unwrap().is_equal_approx(expected));
        assert!(p.intersect_ray(w, v - w).unwrap().is_equal_approx(expected));

        // Vector is perpendicular to normal
        let u = Vector3::new(-3.0, 2.0, 0.0);

        // Test with any points in direction of u (on the plane, not on the plane)
        assert_eq!(p.intersect_ray(p.center(), u), None);
        assert_eq!(p.intersect_ray(v, u), None);
        assert_eq!(p.intersect_ray(w, u), None);
    }

    #[test]
    fn intersects_segment() {
        let (p, v, w) = test_inputs();

        let expected = Vector3::new(1.743063, 4.085646, -0.561722);

        assert!(p.intersect_segment(v, w).unwrap().is_equal_approx(expected));
        assert!(p.intersect_segment(w, v).unwrap().is_equal_approx(expected));

        // Vector is perpendicular to normal
        let u = Vector3::new(-3.0, 2.0, 0.0);
        let pc = p.center();

        // Test with any points in direction of u (on the plane, not on the plane)
        assert_eq!(p.intersect_segment(pc, pc + u), None);
        assert_eq!(p.intersect_segment(v, v + u), None);
        assert_eq!(p.intersect_segment(w, w + u), None);
    }

    #[test]
    fn is_point_over() {
        let (p, v, w) = test_inputs();

        assert!(p.is_point_over(v));

        assert!(!p.is_point_over(w));
        assert!(!p.is_point_over(p.center())); // strictly >
    }

    #[test]
    fn normalized() {
        let (p, ..) = test_inputs();

        let raw = Plane {
            normal: Vector3::new(2.0, 3.0, 7.0),
            d: 11.811012, // = 1.5 * normal.length()
        };

        assert!(raw.normalized().is_equal_approx(p));
    }

    #[test]
    fn is_equal_approx() {
        let (p, ..) = test_inputs();

        // Vectors differing in only d or normal
        let new_d = p.d + 0.1;
        let new_normal = Vector3::new(p.normal.x, p.normal.y, p.normal.z + 0.1).normalized();

        let q1 = Plane::new(p.normal, new_d);
        let q2 = Plane::new(new_normal, p.d);

        assert!(p.is_equal_approx(p));

        assert!(!p.is_equal_approx(q1));
        assert!(!p.is_equal_approx(q2));
    }
}

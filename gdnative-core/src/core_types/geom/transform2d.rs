use crate::core_types::Vector2;

/// Affine 2D transform (2x3 matrix).
///
/// Represents transformations such as translation, rotation, or scaling.
///
/// Expressed as a 2x3 matrix, this transform consists of 2 basis (column) vectors `a` and `b`,
/// as well as an origin `o`; more information in [`Self::from_basis_origin()`]:
/// ```text
/// [ a.x  b.x  o.x ]
/// [ a.y  b.y  o.y ]
/// ```
///
/// Given linear independence, every point in the transformed coordinate system can be expressed as
/// `p = xa + yb + o`, where `x`, `y` are the scaling factors and `o` is the origin.
///
/// See also [Transform2D](https://docs.godotengine.org/en/stable/classes/class_transform2d.html) in the Godot API doc.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Transform2D {
    /// The first basis vector of the transform.
    ///
    /// When transforming the X unit vector `(1, 0)` under this transform, the resulting point is represented by `a`.
    /// Objects will move along `a` when moving on the X axis in the coordinate space of this transform.
    ///
    /// (This field is called `x` in Godot, but was renamed to avoid confusion with the `x` vector component and
    /// less readable expressions such as `x.y`, `y.x`).
    pub a: Vector2,

    /// The second basis vector of the transform.
    ///
    /// When transforming the Y unit vector `(0, 1)` under this transform, the resulting point is represented by `b`.
    /// Objects will move along `b` when moving on the Y axis in the coordinate space of this transform.
    ///
    /// (This field is called `y` in Godot, but was renamed to avoid confusion with the `y` vector component and
    /// less readable expressions such as `x.y`, `y.x`).
    pub b: Vector2,

    /// The origin of the transform. The coordinate space defined by this transform
    /// starts at this point.
    pub origin: Vector2,
}

impl Transform2D {
    /// Represents the identity transform.
    pub const IDENTITY: Self = Self {
        a: Vector2::new(1.0, 0.0),
        b: Vector2::new(0.0, 1.0),
        origin: Vector2::new(0.0, 0.0),
    };

    /// Creates a new transform from three basis vectors and the coordinate system's origin.
    ///
    /// Each vector represents a basis vector in the *transformed* coordinate system.
    /// For example, `a` is the result of transforming the X unit vector `(1, 0)`.
    /// The 2 vectors need to be linearly independent.
    ///
    /// Basis vectors are stored as column vectors in the matrix.
    /// The construction `Transform2D::from_basis_origin(a, b, o)` will create the following 3x4 matrix:
    /// ```text
    /// [ a.x  b.x  o.x ]
    /// [ a.y  b.y  o.y ]
    /// ```
    #[inline]
    pub const fn from_basis_origin(
        basis_vector_a: Vector2,
        basis_vector_b: Vector2,
        origin: Vector2,
    ) -> Self {
        Self {
            a: basis_vector_a,
            b: basis_vector_b,
            origin,
        }
    }

    /// Constructs the transform from a given scale, angle (in radians), and origin.
    ///
    /// This is **NOT** equivalent to either of these two lines:
    /// ```ignore
    /// Transform2D::IDENTITY.scaled(scale).rotated(rotation).translated(origin)
    /// Transform2D::IDENTITY.translated(origin).rotated(rotation).scaled(scale)
    /// ```
    ///
    /// Those transformations do not preserve the given origin; see documentation for [`rotated`], [`scaled`], and [`translated`].
    ///
    /// [`rotated`]: Self::rotated
    /// [`scaled`]: Self::scaled
    /// [`translated`]: Self::translated
    #[inline]
    pub fn from_scale_rotation_origin(scale: Vector2, rotation: f32, origin: Vector2) -> Self {
        let mut tr = Self::IDENTITY;
        tr.set_scale(scale);
        tr.set_rotation(rotation);
        tr.origin = origin;

        tr
    }

    /// Returns the inverse of the transform, under the assumption that the transformation is composed of rotation, scaling and translation.
    #[inline]
    pub fn affine_inverse(&self) -> Self {
        let mut inverted = *self;

        let det = self.basis_determinant();
        debug_assert!(det != 0.0, "The determinant cannot be zero");
        let idet = 1.0 / det;

        std::mem::swap(&mut inverted.a.x, &mut inverted.b.y);
        inverted.a *= Vector2::new(idet, -idet);
        inverted.b *= Vector2::new(-idet, idet);
        inverted.origin = inverted.basis_xform(-inverted.origin);

        inverted
    }

    /// Returns a vector transformed (multiplied) by the basis matrix.
    ///
    /// This method does not account for translation (the origin vector).
    #[inline]
    pub fn basis_xform(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.a.dot(v), self.b.dot(v))
    }

    /// Returns a vector transformed (multiplied) by the inverse basis matrix.
    ///
    /// This method does not account for translation (the origin vector).
    #[inline]
    pub fn basis_xform_inv(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.tdotx(v), self.tdoty(v))
    }

    /// Transforms the given Vector2, Rect2, or PoolVector2Array by this transform.
    #[inline]
    pub fn xform(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.tdotx(v), self.tdoty(v)) + self.origin
    }

    /// Inverse-transforms the given Vector2, Rect2, or PoolVector2Array by this transform.
    #[inline]
    pub fn xform_inv(&self, v: Vector2) -> Vector2 {
        let v = v - self.origin;
        Vector2::new(self.a.dot(v), self.b.dot(v))
    }

    /// Translates the transform by the given offset, relative to the transform's basis vectors.
    ///
    /// Unlike rotated() and scaled(), this does not use matrix multiplication.
    #[inline]
    pub fn translated(&self, translation: Vector2) -> Self {
        Self {
            origin: self.origin + self.basis_xform(translation),
            ..*self
        }
    }

    /// Returns the transform's rotation (in radians).
    #[inline]
    pub fn rotation(&self) -> f32 {
        f32::atan2(self.a.y, self.a.x)
    }

    /// Sets the transform's rotation (argument `rotation` in radians).
    #[inline]
    pub fn set_rotation(&mut self, rotation: f32) {
        let scale = self.scale();
        let cr = f32::cos(rotation);
        let sr = f32::sin(rotation);
        self.a.x = cr;
        self.a.y = sr;
        self.b.x = -sr;
        self.b.y = cr;
        self.set_scale(scale);
    }

    /// Rotates the transform by the given angle (in radians), using matrix multiplication. This will modify the transform's origin.
    #[inline]
    pub fn rotated(&self, rotation: f32) -> Self {
        let mut tr = Self::IDENTITY;
        tr.set_rotation(rotation);
        tr * *self
    }

    /// Returns the transform's scale.
    #[inline]
    pub fn scale(&self) -> Vector2 {
        let det_sign = self.basis_determinant().signum();
        Vector2::new(self.a.length(), det_sign * self.b.length())
    }

    /// Sets the transform's scale.
    #[inline]
    pub fn set_scale(&mut self, scale: Vector2) {
        self.a = self.a.normalized() * scale.x;
        self.b = self.b.normalized() * scale.y;
    }

    /// Scales the transform by the given scale factor, using matrix multiplication. This will modify the transform's origin.
    #[inline]
    pub fn scaled(&self, scale: Vector2) -> Self {
        let mut new = *self;
        new.scale_basis(scale);
        new.origin *= scale;
        new
    }

    /// Returns a transform interpolated between this transform and another by a given weight (on the range of 0.0 to 1.0).
    /// NOTE: This method assumes both Transform2Ds are affine transformations.
    #[inline]
    pub fn interpolate_with(&self, other: Self, weight: f32) -> Self {
        // extract parameters
        let p1 = self.origin;
        let p2 = other.origin;

        let r1 = self.rotation();
        let r2 = other.rotation();

        let s1 = self.scale();
        let s2 = other.scale();

        // slerp rotation
        let v1 = Vector2::new(f32::cos(r1), f32::sin(r1));
        let v2 = Vector2::new(f32::cos(r2), f32::sin(r2));
        let dot = v1.dot(v2).clamp(-1.0, 1.0);

        let v = if dot > 0.9995 {
            //linearly interpolate to avoid numerical precision issues
            v1.linear_interpolate(v2, weight).normalized()
        } else {
            let angle = weight * f32::cos(dot);
            let v3 = (v2 - v1 * dot).normalized();
            v1 * f32::cos(angle) + v3 * f32::sin(angle)
        };

        // construct matrix
        let mut result = Self::IDENTITY
            .rotated(f32::atan2(v.y, v.x))
            .translated(p1.linear_interpolate(p2, weight));
        result.scale_basis(s1.linear_interpolate(s2, weight));
        result
    }

    /// Returns true if this transform and transform are approximately equal, by calling is_equal_approx on each component.
    #[inline]
    pub fn is_equal_approx(&self, other: Transform2D) -> bool {
        self.a.is_equal_approx(other.a)
            && self.b.is_equal_approx(other.b)
            && self.origin.is_equal_approx(other.origin)
    }

    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_transform2d {
        self as *const _ as *const _
    }

    /// Internal API for converting to `sys` representation. Makes it possible to remove
    /// `transmute`s elsewhere.
    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut sys::godot_transform2d {
        self as *mut _ as *mut _
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(c: sys::godot_transform2d) -> Self {
        unsafe { std::mem::transmute::<sys::godot_transform2d, Self>(c) }
    }

    fn basis_determinant(&self) -> f32 {
        self.a.x * self.b.y - self.a.y * self.b.x
    }

    fn tdotx(&self, v: Vector2) -> f32 {
        self.a.x * v.x + self.b.x * v.y
    }

    fn tdoty(&self, v: Vector2) -> f32 {
        self.a.y * v.x + self.b.y * v.y
    }

    fn scale_basis(&mut self, scale: Vector2) {
        self.a.x *= scale.x;
        self.a.y *= scale.y;
        self.b.x *= scale.x;
        self.b.y *= scale.y;
    }
}

impl std::ops::Mul<Transform2D> for Transform2D {
    type Output = Transform2D;

    #[inline]
    fn mul(self, rhs: Transform2D) -> Self::Output {
        let mut new = self;

        new.origin = new.xform(rhs.origin);

        let x0 = new.tdotx(rhs.a);
        let x1 = new.tdoty(rhs.a);
        let y0 = new.tdotx(rhs.b);
        let y1 = new.tdoty(rhs.b);

        new.a.x = x0;
        new.a.y = x1;
        new.b.x = y0;
        new.b.y = y1;

        new
    }
}

#[cfg(feature = "gd-test")]
fn test_transform2d_behavior_impl() {
    let api = crate::private::get_api();

    // This test compares the Transform2D implementation against the Godot API,
    // making sure behavior is consistent between the two.

    let new_transform_rust = Transform2D::from_basis_origin(
        Vector2::new(42.0, 0.0),
        Vector2::new(0.0, 23.0),
        Vector2::new(5.0, 8.0),
    );

    // constructors yield same results

    let new_transform_godot = {
        let mut tr = Transform2D::IDENTITY;
        let x_axis = Vector2::new(42.0, 0.0);
        let y_axis = Vector2::new(0.0, 23.0);
        let origin = Vector2::new(5.0, 8.0);
        unsafe {
            (api.godot_transform2d_new_axis_origin)(
                tr.sys_mut(),
                x_axis.sys(),
                y_axis.sys(),
                origin.sys(),
            );
        }
        tr
    };

    assert_eq!(
        new_transform_rust, new_transform_godot,
        "Newly constructed transforms should be identical"
    );

    // Affine inverse

    let rust_inverse = new_transform_rust.affine_inverse();
    let godot_inverse = Transform2D::from_sys(unsafe {
        (api.godot_transform2d_affine_inverse)(new_transform_rust.sys())
    });

    assert!(
        rust_inverse.is_equal_approx(godot_inverse),
        "Affine inverse operation should yield equal results"
    );

    // Translation, rotation, scale

    let translation_vector = Vector2::new(3.0, 6.0);
    let rotation_angle = std::f32::consts::FRAC_PI_2;
    let scale_vector = Vector2::new(7.0, 9.0);

    let transformed_rust = new_transform_rust
        .translated(translation_vector)
        .rotated(rotation_angle)
        .scaled(scale_vector);

    let transformed_godot = {
        let tr1 = Transform2D::from_sys(unsafe {
            (api.godot_transform2d_translated)(new_transform_godot.sys(), translation_vector.sys())
        });
        let tr2 = Transform2D::from_sys(unsafe {
            (api.godot_transform2d_rotated)(tr1.sys(), rotation_angle)
        });
        Transform2D::from_sys(unsafe {
            (api.godot_transform2d_scaled)(tr2.sys(), scale_vector.sys())
        })
    };

    assert!(
        transformed_rust.is_equal_approx(transformed_godot),
        "Transformations should yield equal results"
    );

    let rotation_rust = new_transform_rust.rotation();
    let rotation_godot = unsafe { (api.godot_transform2d_get_rotation)(new_transform_rust.sys()) };

    approx::assert_relative_eq!(rotation_rust, rotation_godot);

    let scale_rust = new_transform_rust.scale();
    let scale_godot =
        Vector2::from_sys(unsafe { (api.godot_transform2d_get_scale)(new_transform_rust.sys()) });

    assert!(
        scale_rust.is_equal_approx(scale_godot),
        "Scale getters should return equal results"
    );

    let other_transform_rust = Transform2D::from_basis_origin(
        Vector2::new(10.0, 0.0),
        Vector2::new(0.0, 15.0),
        Vector2::new(5.0, 13.0),
    );
    let interpolation_weight = 0.35;

    let interpolated_rust =
        new_transform_rust.interpolate_with(other_transform_rust, interpolation_weight);
    let interpolated_godot = {
        Transform2D::from_sys(unsafe {
            (api.godot_transform2d_interpolate_with)(
                new_transform_rust.sys(),
                other_transform_rust.sys(),
                interpolation_weight,
            )
        })
    };

    assert!(
        interpolated_rust.is_equal_approx(interpolated_godot),
        "Transform2D interpolation should yield equal results"
    );

    let some_vec = Vector2::new(18.0, 15.0);
    let basis_xformed_rust = new_transform_rust.basis_xform(some_vec);
    let basis_xformed_godot = Vector2::from_sys(unsafe {
        (api.godot_transform2d_basis_xform_vector2)(new_transform_rust.sys(), some_vec.sys())
    });

    assert!(
        basis_xformed_rust.is_equal_approx(basis_xformed_godot),
        "Transformed vectors using basis should be equal"
    );

    let basis_xformed_inv_rust = new_transform_rust.basis_xform_inv(some_vec);
    let basis_xformed_inv_godot = Vector2::from_sys(unsafe {
        (api.godot_transform2d_basis_xform_inv_vector2)(new_transform_rust.sys(), some_vec.sys())
    });

    assert!(
        basis_xformed_inv_rust.is_equal_approx(basis_xformed_inv_godot),
        "Transformed vectors using basis should be equal"
    );
}

godot_test!(
    test_transform2d_behavior {
        test_transform2d_behavior_impl()
    }
);

#[test]
fn test_transform2d_constructor() {
    use std::f32::consts::PI;

    let scale = Vector2::new(2.0, 0.5);
    let rotation = PI / 4.0;
    let origin = Vector2::new(250.0, 150.0);

    let tr = Transform2D::from_scale_rotation_origin(scale, rotation, origin);

    assert_eq!(tr.origin, origin);

    let actual_local_right = tr.basis_xform(Vector2::RIGHT);
    let expected_local_right = Vector2::RIGHT.rotated(-rotation) * scale;
    assert!(
        actual_local_right.is_equal_approx(expected_local_right),
        "{actual_local_right:?} != {expected_local_right:?}"
    );
}

use super::Vector2;

/// Clamp method for f32.
/// NOTE: This method was copied as-is from std. This was done to avoid compatibility issues
/// with newer rustc versions and should be removed in favor of f32::clamp once that is stable.
#[inline]
fn clamp(num: f32, min: f32, max: f32) -> f32 {
    assert!(min <= max);
    let mut x = num;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

/// A 2×3 matrix (2 rows, 3 columns) used for 2D linear transformations. It can represent
/// transformations such as translation, rotation, or scaling. It consists of three
/// Vector2 values: x axis, y axis, and the origin.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Transform2D {
    /// The x basis vector of the transform. Objects will move along this vector when
    /// moving on the X axis in the coordinate space of this transform
    pub x: Vector2,
    /// The y basis vector of the transform. Objects will move along this vector when
    /// moving on the Y axis in the coordinate space of this transform.
    pub y: Vector2,
    /// The origin of the transform. The coordinate space defined by this transform
    /// starts at this point.
    pub origin: Vector2,
}

impl Transform2D {
    fn basis_determinant(&self) -> f32 {
        self.x.x * self.y.y - self.x.y * self.y.x
    }

    fn tdotx(&self, v: Vector2) -> f32 {
        self.x.x * v.x + self.y.x * v.y
    }

    fn tdoty(&self, v: Vector2) -> f32 {
        self.x.y * v.x + self.y.y * v.y
    }

    /// Represents the identity transform.
    pub const IDENTITY: Self = Self {
        x: Vector2::new(1.0, 0.0),
        y: Vector2::new(0.0, 1.0),
        origin: Vector2::new(0.0, 0.0),
    };

    /// Constructs the transform from 3 Vector2 values representing x, y, and the origin (the three column vectors).
    #[inline]
    pub fn from_axis_origin(x_axis: Vector2, y_axis: Vector2, origin: Vector2) -> Transform2D {
        Transform2D {
            x: x_axis,
            y: y_axis,
            origin,
        }
    }

    /// Constructs the transform from a given angle (in radians), translation, and scale.
    #[inline]
    pub fn from_rotation_translation_scale(
        translation: Vector2,
        rotation: f32,
        scale: Vector2,
    ) -> Transform2D {
        Self::IDENTITY
            .translated(translation)
            .rotated(rotation)
            .scaled(scale)
    }

    /// Returns the inverse of the transform, under the assumption that the transformation is composed of rotation, scaling and translation.
    #[inline]
    pub fn affine_inverse(&self) -> Transform2D {
        let mut inverted = *self;

        let det = self.basis_determinant();
        debug_assert!(det != 0.0, "The determinant cannot be zero");
        let idet = 1.0 / det;

        std::mem::swap(&mut inverted.x.x, &mut inverted.y.y);
        inverted.x *= Vector2::new(idet, -idet);
        inverted.y *= Vector2::new(-idet, idet);
        inverted.origin = inverted.basis_xform(-inverted.origin);

        inverted
    }

    /// Returns a vector transformed (multiplied) by the basis matrix.
    ///
    /// This method does not account for translation (the origin vector).
    #[inline]
    pub fn basis_xform(&self, v: Vector2) -> Vector2 {
        Vector2::new(self.x.dot(v), self.y.dot(v))
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
        Vector2::new(self.x.dot(v), self.y.dot(v))
    }

    /// Translates the transform by the given offset, relative to the transform's basis vectors.
    ///
    /// Unlike rotated() and scaled(), this does not use matrix multiplication.
    #[inline]
    pub fn translated(&self, translation: Vector2) -> Transform2D {
        Self {
            origin: self.origin + self.basis_xform(translation),
            ..*self
        }
    }

    /// Returns the transform's rotation (in radians).
    #[inline]
    pub fn rotation(&self) -> f32 {
        f32::atan2(self.x.y, self.x.x)
    }

    /// Sets the transform's rotation (argument `rotation` in radians).
    #[inline]
    pub fn set_rotation(&mut self, rotation: f32) {
        let scale = self.scale();
        let cr = f32::cos(rotation);
        let sr = f32::sin(rotation);
        self.x.x = cr;
        self.x.y = sr;
        self.y.x = -sr;
        self.y.y = cr;
        self.set_scale(scale);
    }

    /// Rotates the transform by the given angle (in radians), using matrix multiplication.
    #[inline]
    pub fn rotated(&self, rotation: f32) -> Transform2D {
        let mut tr = Transform2D::IDENTITY;
        tr.set_rotation(rotation);
        tr * *self
    }

    /// Returns the transform's scale.
    #[inline]
    pub fn scale(&self) -> Vector2 {
        let det_sign = self.basis_determinant().signum();
        Vector2::new(self.x.length(), det_sign * self.y.length())
    }

    /// Sets the transform's scale.
    #[inline]
    pub fn set_scale(&mut self, scale: Vector2) {
        self.x = self.x.normalized() * scale.x;
        self.y = self.y.normalized() * scale.y;
    }

    /// Scales the transform by the given scale factor, using matrix multiplication.
    #[inline]
    pub fn scaled(&self, scale: Vector2) -> Transform2D {
        let mut new = *self;
        new.scale_basis(scale);
        new.origin *= scale;
        new
    }

    fn scale_basis(&mut self, scale: Vector2) {
        self.x.x *= scale.x;
        self.x.y *= scale.y;
        self.y.x *= scale.x;
        self.y.y *= scale.y;
    }

    /// Returns a transform interpolated between this transform and another by a given weight (on the range of 0.0 to 1.0).
    /// NOTE: This method assumes both Transform2Ds are affine transformations.
    #[inline]
    pub fn interpolate_with(&self, other: Transform2D, weight: f32) -> Transform2D {
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
        let dot = clamp(v1.dot(v2), -1.0, 1.0);

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
        self.x.is_equal_approx(other.x)
            && self.y.is_equal_approx(other.y)
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
}

impl std::ops::Mul<Transform2D> for Transform2D {
    type Output = Transform2D;

    #[inline]
    fn mul(self, rhs: Transform2D) -> Self::Output {
        let mut new = self;

        new.origin = new.xform(rhs.origin);

        let x0 = new.tdotx(rhs.x);
        let x1 = new.tdoty(rhs.x);
        let y0 = new.tdotx(rhs.y);
        let y1 = new.tdoty(rhs.y);

        new.x.x = x0;
        new.x.y = x1;
        new.y.x = y0;
        new.y.y = y1;

        new
    }
}

fn test_transform2d_behavior_impl() {
    let api = crate::private::get_api();

    // This test compares the Transform2D implementation against the Godot API,
    // making sure behavior is consistent between the two.

    let new_transform_rust = Transform2D::from_axis_origin(
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

    assert!(
        (rotation_rust - rotation_godot) < f32::EPSILON,
        "Rotation getters should return equal results"
    );

    let scale_rust = new_transform_rust.scale();
    let scale_godot =
        Vector2::from_sys(unsafe { (api.godot_transform2d_get_scale)(new_transform_rust.sys()) });

    assert!(
        scale_rust.is_equal_approx(scale_godot),
        "Scale getters should return equal results"
    );

    let other_transform_rust = Transform2D::from_axis_origin(
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

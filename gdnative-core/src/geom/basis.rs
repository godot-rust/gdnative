use crate::Vector3;
use euclid::{default, Transform3D, UnknownUnit, Vector3D};

/// A 3x3 matrix.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Basis {
    pub elements: [Vector3; 3],
}

impl Basis {
    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_basis {
        unsafe { std::mem::transmute::<*const Basis, *const sys::godot_basis>(self as *const _) }
    }

    #[doc(hidden)]
    pub fn from_sys(c: sys::godot_basis) -> Self {
        unsafe { std::mem::transmute::<sys::godot_basis, Self>(c) }
    }

    pub fn identity() -> Basis {
        Basis {
            elements: [
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ],
        }
    }

    pub fn from_diagonal(p_diag: Vector3) -> Basis {
        Basis {
            elements: [
                Vector3::new(p_diag.x, 0.0, 0.0),
                Vector3::new(0.0, p_diag.y, 0.0),
                Vector3::new(0.0, 0.0, p_diag.z),
            ],
        }
    }

    /// Creates a `Basis` from the rotation and scaling of the provided transform.
    pub fn from_transform(transform: &default::Transform3D<f32>) -> Basis {
        Self::from_typed_transform::<UnknownUnit, UnknownUnit>(transform)
    }

    /// Creates a `Basis` from the rotation and scaling of the provided transform, in `Dst` space.
    pub fn from_typed_transform<Src, Dst>(transform: &Transform3D<f32, Src, Dst>) -> Basis {
        Basis {
            elements: [
                transform
                    .transform_vector3d(Vector3D::<_, Src>::new(1.0, 0.0, 0.0))
                    .to_untyped(),
                transform
                    .transform_vector3d(Vector3D::<_, Src>::new(0.0, 1.0, 0.0))
                    .to_untyped(),
                transform
                    .transform_vector3d(Vector3D::<_, Src>::new(0.0, 0.0, 1.0))
                    .to_untyped(),
            ],
        }
    }

    /// Transposed dot product with the x axis of the matrix.
    pub fn tdotx(&self, v: Vector3) -> f32 {
        self.elements[0].x * v.x + self.elements[1].x * v.y + self.elements[2].x * v.z
    }

    /// Transposed dot product with the y axis of the matrix.
    pub fn tdoty(&self, v: Vector3) -> f32 {
        self.elements[0].y * v.x + self.elements[1].y * v.y + self.elements[2].y * v.z
    }

    /// Transposed dot product with the z axis of the matrix.
    pub fn tdotz(&self, v: Vector3) -> f32 {
        self.elements[0].z * v.x + self.elements[1].z * v.y + self.elements[2].z * v.z
    }

    /// Get the x axis of the matrix
    pub fn x(&self) -> Vector3 {
        Vector3::new(self.elements[0].x, self.elements[1].x, self.elements[2].x)
    }

    /// Set the x axis of the matrix
    pub fn set_x(&mut self, v: Vector3) {
        self.elements[0].x = v.x;
        self.elements[1].x = v.y;
        self.elements[2].x = v.z;
    }

    /// Get the y axis of the matrix
    pub fn y(&self) -> Vector3 {
        Vector3::new(self.elements[0].y, self.elements[1].y, self.elements[2].y)
    }

    /// Set the y axis of the matrix
    pub fn set_y(&mut self, v: Vector3) {
        self.elements[0].y = v.x;
        self.elements[1].y = v.y;
        self.elements[2].y = v.z;
    }

    /// Get the z axis of the matrix
    pub fn z(&self) -> Vector3 {
        Vector3::new(self.elements[0].z, self.elements[1].z, self.elements[2].z)
    }

    /// Set the z access of the matrix
    pub fn set_z(&mut self, v: Vector3) {
        self.elements[0].z = v.x;
        self.elements[1].z = v.y;
        self.elements[2].z = v.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transposed_dot_is_sane() {
        let basis = Basis {
            elements: [
                Vector3::new(1.0, 2.0, 3.0),
                Vector3::new(2.0, 3.0, 4.0),
                Vector3::new(3.0, 4.0, 5.0),
            ],
        };

        let vector = Vector3::new(4.0, 5.0, 6.0);

        assert!((basis.tdotx(vector) - 32.0).abs() < std::f32::EPSILON);
        assert!((basis.tdoty(vector) - 47.0).abs() < std::f32::EPSILON);
        assert!((basis.tdotz(vector) - 62.0).abs() < std::f32::EPSILON);
    }

    #[test]
    fn retrieval_is_sane() {
        let basis = Basis {
            elements: [
                Vector3::new(1.0, 2.0, 3.0),
                Vector3::new(4.0, 5.0, 6.0),
                Vector3::new(7.0, 8.0, 9.0),
            ],
        };

        assert!(basis.x() == Vector3::new(1.0, 4.0, 7.0));
        assert!(basis.y() == Vector3::new(2.0, 5.0, 8.0));
        assert!(basis.z() == Vector3::new(3.0, 6.0, 9.0));
    }

    #[test]

    fn set_is_sane() {
        let mut basis = Basis {
            elements: [Vector3::zero(), Vector3::zero(), Vector3::zero()],
        };

        basis.set_x(Vector3::new(1.0, 4.0, 7.0));
        basis.set_y(Vector3::new(2.0, 5.0, 8.0));
        basis.set_z(Vector3::new(3.0, 6.0, 9.0));

        assert!(basis.elements[0] == Vector3::new(1.0, 2.0, 3.0));
        assert!(basis.elements[1] == Vector3::new(4.0, 5.0, 6.0));
        assert!(basis.elements[2] == Vector3::new(7.0, 8.0, 9.0));
    }
}

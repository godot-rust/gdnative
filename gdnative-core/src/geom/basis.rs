use crate::Vector3;
use euclid::{Transform3D, Vector3D};

/// A 3x3 matrix.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Basis {
    pub elements: [Vector3; 3],
}

// TODO more methods!
// Feel free to get inspiration from godot-src\core\math\basis.cpp
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

    // Note: It's ok to use euclid::UnknownUnit as the type for both Src and Dst
    pub fn from_transform<Src, Dst>(transform: &Transform3D<f32, Src, Dst>) -> Basis {
        // Note - this encodes just the rotation and scaling
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
            ]
        };

        let vector = Vector3::new(4.0, 5.0, 6.0);

	    assert!((basis.tdotx(vector) - 32.0).abs() < std::f32::EPSILON);
	    assert!((basis.tdoty(vector) - 47.0).abs() < std::f32::EPSILON);
	    assert!((basis.tdotz(vector) - 62.0).abs() < std::f32::EPSILON);
    }
}
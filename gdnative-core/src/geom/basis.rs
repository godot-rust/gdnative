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
    /// A transform which does nothing.
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

    /// First create a transform using euclid and then turn it into a godot transform at the last
    ///  possible moment.
    ///
    /// Note: It's ok to use euclid::UnknownUnit as the type for both Src and Dst
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

    // transposed dot products
    fn tdotx(&self, v: Vector3) -> f32 {
        // Note: This is a port of the Godot C++ code and is not well tested yet in Rust.
        self.elements[0].x * v.x + self.elements[1].x * v.y + self.elements[2].x * v.z
    }

    fn tdoty(&self, v: Vector3) -> f32 {
        // Note: This is a port of the Godot C++ code and is not well tested yet in Rust.
        self.elements[0].y * v.x + self.elements[1].y * v.y + self.elements[2].y * v.z
    }

    fn tdotz(&self, v: Vector3) -> f32 {
        // Note: This is a port of the Godot C++ code and is not well tested yet in Rust.
        self.elements[0].z * v.x + self.elements[1].z * v.y + self.elements[2].z * v.z
    }
}

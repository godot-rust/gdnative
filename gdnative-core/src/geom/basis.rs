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
    //
    //    /// set_euler_yxz expects a vector containing the Euler angles in the format
    //    /// (ax,ay,az), where ax is the angle of rotation around x axis,
    //    /// and similar for other axes.
    //    /// The current implementation uses YXZ convention (Z is the first rotation).
    //    pub fn from_euler_yxz(p_euler: Vector3) -> Basis {
    //
    //        let c = Math::cos(p_euler.x);
    //        let s = Math::sin(p_euler.x);
    //        let xmat = Basis::element_new(1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c);
    //
    //        let c = Math::cos(p_euler.y);
    //        let s = Math::sin(p_euler.y);
    //        let ymat = Basis::element_new(c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c);
    //
    //        let c = Math::cos(p_euler.z);
    //        let s = Math::sin(p_euler.z);
    //        let zmat = Basis::element_new(c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0);
    //
    //        // optimizer will optimize away all this anyway
    //        ymat * xmat * zmat;
    //    }
    //
    //    // transposed dot products
    //    fn tdotx(&self, v: Vector3) -> f32 {
    //        self.elements[0].x * v[0] + self.elements[1].x * v[1] + self.elements[2].x * v[2]
    //    }
    //    fn tdoty(&self, v: Vector3) -> f32 {
    //        self.elements[0].y * v[0] + self.elements[1].y * v[1] + self.elements[2].y * v[2]
    //    }
    //    fn tdotz(&self, v: Vector3) -> f32 {
    //        self.elements[0].z * v[0] + self.elements[1].z * v[1] + self.elements[2].z * v[2]
    //    }
}

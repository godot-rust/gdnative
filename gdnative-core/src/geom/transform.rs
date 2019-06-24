use crate::{Basis, Vector3};

/// 3D Transformation (3x4 matrix) Using basis + origin representation.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    /// The basis is a matrix containing 3 Vector3 as its columns: X axis, Y axis, and Z axis.
    /// These vectors can be interpreted as the basis vectors of local coordinate system
    /// traveling with the object.
    pub basis: Basis,
    /// The translation offset of the transform.
    pub origin: Vector3,
}

// TODO: methods!


use Vector3;

/// Axis-aligned bounding box.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Aabb {
    pub position: Vector3,
    pub size: Vector3,
}

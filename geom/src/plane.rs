use Vector3;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

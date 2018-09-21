use Vector3;

/// Plane in hessian form.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

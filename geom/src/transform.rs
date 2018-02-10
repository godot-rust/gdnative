use {Vector3, Basis};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub basis: Basis,
    pub origin: Vector3,
}

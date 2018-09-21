use Vector3;

/// A 3x3 matrix.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Basis {
    pub elements: [Vector3; 3],
}

// TODO methods!

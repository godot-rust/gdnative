use super::Vector2;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Transform2D {
    pub x: Vector2,
    pub y: Vector2,
    pub origin: Vector2,
}

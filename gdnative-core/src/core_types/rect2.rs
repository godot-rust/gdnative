use super::Vector2;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Rect2 {
    pub position: Vector2,
    pub size: Vector2,
}

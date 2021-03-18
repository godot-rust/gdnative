use super::Vector2;

#[repr(C)]
pub struct Rect2 {
    pub position: Vector2,
    pub size: Vector2,
}

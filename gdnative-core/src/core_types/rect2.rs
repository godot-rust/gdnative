use super::Vector2;

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Rect2 {
    pub position: Vector2,
    pub size: Vector2,
}

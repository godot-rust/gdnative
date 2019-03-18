//! Geometric types.

mod plane;
mod aabb;
mod basis;
mod transform;

pub type Vector3 = euclid::Vector3D<f32>;
pub type Vector2 = euclid::Vector2D<f32>;
pub type Transform2D = euclid::Transform2D<f32>;
pub type Quat = euclid::Rotation3D<f32>;
pub type Rect2 = euclid::Rect<f32>;

pub use crate::plane::Plane;
pub use crate::aabb::Aabb;
pub use crate::basis::Basis;
pub use crate::transform::Transform;

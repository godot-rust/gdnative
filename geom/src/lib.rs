//! Geometric types.

pub extern crate euclid;

mod plane;
mod aabb;
mod basis;
mod transform;

pub type Vector3 = euclid::Vector3D<f32>;
pub type Vector2 = euclid::Vector2D<f32>;
pub type Transform2D = euclid::Transform2D<f32>;
pub type Quat = euclid::Rotation3D<f32>;
pub type Rect2 = euclid::Rect<f32>;

pub use plane::Plane;
pub use aabb::Aabb;
pub use basis::Basis;
pub use transform::Transform;

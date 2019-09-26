//! Geometric types.

mod aabb;
mod basis;
mod plane;
mod transform;

pub type Vector3 = euclid::Vector3D<f32, euclid::UnknownUnit>;
pub type Vector2 = euclid::Vector2D<f32, euclid::UnknownUnit>;
pub type Transform2D = euclid::Transform2D<f32, euclid::UnknownUnit, euclid::UnknownUnit>;
pub type Quat = euclid::Rotation3D<f32, euclid::UnknownUnit, euclid::UnknownUnit>;
pub type Rect2 = euclid::Rect<f32, euclid::UnknownUnit>;

pub use self::aabb::Aabb;
pub use self::basis::Basis;
pub use self::plane::Plane;
pub use self::transform::Transform;

//! Geometric types.

mod aabb;
mod basis;
mod plane;
mod transform;

pub type Vector3 = euclid::default::Vector3D<f32>;
pub type Vector2 = euclid::default::Vector2D<f32>;
pub type Transform2D = euclid::default::Transform2D<f32>;
pub type Quat = euclid::default::Rotation3D<f32>;
pub type Size2 = euclid::default::Size2D<f32>;
pub type Rect2 = euclid::default::Rect<f32>;
pub type Angle = euclid::Angle<f32>;
pub type Point3 = euclid::default::Point3D<f32>;
pub type Point2 = euclid::default::Point2D<f32>;
pub type Rotation2D = euclid::default::Rotation2D<f32>;
pub type Rotation3D = euclid::default::Rotation3D<f32>;

pub use self::aabb::Aabb;
pub use self::basis::Basis;
pub use self::plane::Plane;
pub use self::transform::Transform;

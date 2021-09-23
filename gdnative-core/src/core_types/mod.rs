//! Types that represent core-datatypes of Godot.

mod geom;

mod access;
mod byte_array;
mod color;
mod color_array;
mod float32_array;
mod int32_array;
mod node_path;
mod quat;
mod rect2;
mod rid;
mod string;
mod string_array;
mod transform2d;
mod typed_array;
mod variant;
mod variant_array;
mod vector2;
mod vector2_array;
mod vector3_array;

pub mod dictionary;
pub mod error;
pub mod vector3;

pub use geom::*;

pub use access::*;
pub use byte_array::*;
pub use color::*;
pub use color_array::*;
pub use dictionary::Dictionary;
pub use error::GodotError;
pub use float32_array::*;
pub use int32_array::*;
pub use node_path::*;
pub use quat::*;
pub use rect2::*;
pub use rid::*;
pub use string::*;
pub use string_array::*;
pub use transform2d::*;
pub use typed_array::{Element, TypedArray};
pub use variant::*;
pub use variant_array::*;
pub use vector2::*;
pub use vector2_array::*;
pub use vector3::*;
pub use vector3_array::*;

use approx::relative_eq;

const CMP_EPSILON: f64 = 0.00001;

// This trait is intended for internal use
trait IsEqualApprox {
    #[allow(clippy::wrong_self_convention)]
    fn is_equal_approx(self, to: Self) -> bool;
}

impl IsEqualApprox for f32 {
    fn is_equal_approx(self, to: Self) -> bool {
        relative_eq!(self, to, epsilon = CMP_EPSILON as f32)
    }
}

impl IsEqualApprox for f64 {
    fn is_equal_approx(self, to: Self) -> bool {
        relative_eq!(self, to, epsilon = CMP_EPSILON)
    }
}

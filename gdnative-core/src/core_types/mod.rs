//! Types that represent [core types](https://docs.godotengine.org/en/stable/development/cpp/core_types.html) of Godot.
//!
//! In contrast to generated Godot class types from the `api` module, the types in here are hand-written in idiomatic Rust and
//! are the counterparts to built-in types in GDScript.
//!
//! godot-rust provides optional serialization support for many core types.  Enable the feature `serde` to make use of it.

mod geom;

mod access;
mod color;
mod dictionary;
mod error;
mod node_path;
mod pool_array;
mod rid;
mod string;
mod variant;
mod variant_array;
mod vector2;
mod vector3;

pub use access::*;
pub use color::*;
pub use dictionary::*;
pub use error::{GodotError, GodotResult};
pub use geom::*;
pub use node_path::*;
pub use pool_array::*;
pub use rid::*;
pub use string::*;
pub use variant::*;
pub use variant_array::*;
pub use vector2::*;
pub use vector3::*;

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

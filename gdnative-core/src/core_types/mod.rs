//! Types that represent [core types](https://docs.godotengine.org/en/stable/development/cpp/core_types.html) of Godot.
//!
//! In contrast to generated Godot class types from the `api` module, the types in here are hand-written in idiomatic Rust and
//! are the counterparts to built-in types in GDScript.
//!
//! godot-rust provides optional serialization support for many core types.  Enable the feature `serde` to make use of it.

mod color;
mod error;
mod node_path;
mod pool_array;
mod rid;
mod vector2;
mod vector3;

pub mod access;
pub mod array;
pub mod dictionary;
pub mod geom;
pub mod string;
pub mod variant;

pub use array::VariantArray;
pub use color::Color;
pub use dictionary::Dictionary;
pub use error::{GodotError, GodotResult};
pub use geom::{Aabb, Basis, Margin, MarginError, Plane, Quat, Rect2, Transform, Transform2D};
pub use node_path::NodePath;
pub use pool_array::{PoolArray, PoolElement};
pub use rid::Rid;
pub use string::{GodotString, StringName};
pub use variant::{
    CoerceFromVariant, FromVariant, FromVariantError, OwnedToVariant, ToVariant, ToVariantEq,
    Variant, VariantType,
};
pub use vector2::Vector2;
pub use vector3::{Axis, Vector3};

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

#[cfg(feature = "gd-test")]
#[doc(hidden)]
#[inline]
#[must_use]
pub fn test_core_types() -> bool {
    let mut status = true;

    status &= string::test_string();
    status &= string::test_string_name_eq();
    status &= string::test_string_name_ord();

    status &= array::test_array();
    status &= array::test_array_debug();
    status &= array::test_array_clone_clear();
    status &= dictionary::test_dictionary();
    status &= dictionary::test_dictionary_clone_clear();

    status &= color::test_color();
    status &= vector2::test_vector2_variants();
    status &= vector3::test_vector3_variants();

    status &= variant::test_variant_nil();
    status &= variant::test_variant_i64();
    status &= variant::test_variant_bool();
    status &= variant::test_variant_option();
    status &= variant::test_variant_result();
    status &= variant::test_variant_hash_map();
    status &= variant::test_variant_hash_set();
    status &= variant::test_variant_vec();
    status &= variant::test_to_variant_iter();
    status &= variant::test_variant_tuple();
    status &= variant::test_variant_dispatch();

    status &= pool_array::test_byte_array_access();
    status &= pool_array::test_int32_array_access();
    status &= pool_array::test_float32_array_access();
    status &= pool_array::test_color_array_access();
    status &= pool_array::test_string_array_access();
    status &= pool_array::test_vector2_array_access();
    status &= pool_array::test_vector3_array_access();

    status &= geom::test_transform2d_behavior();

    status
}

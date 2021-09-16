use crate::*;
use std::default::Default;
use std::fmt;
use std::mem::{forget, transmute};
use std::ptr;

use crate::core_types::*;
use crate::object::*;
use crate::private::{get_api, ManuallyManagedClassPlaceholder};
use crate::thread_access::*;

// TODO: implement Debug, PartialEq, etc.

/// A `Variant` can represent many of godot's core types.
///
/// The underlying data can be either stored inline or reference-counted,
/// dependning on the size of the type and whether the it is trivially copyable.
pub struct Variant(pub(crate) sys::godot_variant);

macro_rules! variant_constructors {
    (
        $(
            $(#[$attr:meta])*
            pub fn $ctor:ident($Type:ty) -> Self;
        )*
    ) => (
        $(
            $(#[$attr])*
            pub fn $ctor(val: $Type) -> Variant {
                ToVariant::to_variant(val)
            }
        )*
    )
}

macro_rules! variant_to_type_transmute {
    (
        $(
            $(#[$to_attr:meta])*
            pub fn $to_method:ident(&self) -> $ToType:ident : $to_gd_method:ident;
            $(#[$try_attr:meta])*
            pub fn $try_method:ident(&self) -> Option<$TryType:ident>;
        )*
    ) => (
        $(
            $(#[$to_attr])*
            pub fn $to_method(&self) -> $ToType {
                unsafe {
                    #[allow(clippy::useless_transmute)]
                    transmute((get_api().$to_gd_method)(&self.0))
                }
            }

            $(#[$try_attr])*
            pub fn $try_method(&self) -> Option<$TryType> {
                $TryType::from_variant(self).ok()
            }
        )*
    )
}

macro_rules! variant_to_type_from_sys {
    (
        $(
            $(#[$to_attr:meta])*
            pub fn $to_method:ident(&self) -> $ToType:ty : $to_gd_method:ident;
            $(#[$try_attr:meta])*
            pub fn $try_method:ident(&self) -> Option<$TryType:ty>;
        )*
    ) => (
        $(
            $(#[$to_attr])*
            pub fn $to_method(&self) -> $ToType {
                unsafe {
                    <$ToType>::from_sys((get_api().$to_gd_method)(&self.0))
                }
            }

            $(#[$try_attr])*
            pub fn $try_method(&self) -> Option<$TryType> {
                <$TryType>::from_variant(self).ok()
            }
        )*
    )
}

macro_rules! variant_dispatch_arm {
    ($v:expr, $variant:ident ( $inner:ty )) => {
        VariantDispatch::$variant(<$inner>::from_variant($v).unwrap())
    };
    ($v:expr, $variant:ident) => {
        VariantDispatch::$variant
    };
}

macro_rules! decl_variant_type {
    (
        pub enum VariantType, VariantDispatch {
            $(
                $variant:ident $( ($inner:ty) )? = $c_const:path,
            )*
        }
    ) => {
        #[repr(u32)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        pub enum VariantType {
            $(
                $variant = $c_const as u32,
            )*
        }

        /// Rust enum associating each primitive variant type to its value.
        ///
        /// For `Variant`s containing objects, the original `Variant` is returned unchanged, due to
        /// the limitations of statically-determined memory management.
        #[derive(Clone, Debug)]
        #[repr(u32)]
        pub enum VariantDispatch {
            $(
                $variant $( ($inner) )?,
            )*
        }

        impl<'a> From<&'a Variant> for VariantDispatch {
            #[inline]
            fn from(v: &'a Variant) -> Self {
                match v.get_type() {
                    $(
                        VariantType::$variant => {
                            variant_dispatch_arm!(v, $variant $( ($inner) )?)
                        },
                    )*
                }
            }
        }
    }
}

decl_variant_type!(
    pub enum VariantType, VariantDispatch {
        Nil = sys::godot_variant_type_GODOT_VARIANT_TYPE_NIL,
        Bool(bool) = sys::godot_variant_type_GODOT_VARIANT_TYPE_BOOL,
        I64(i64) = sys::godot_variant_type_GODOT_VARIANT_TYPE_INT,
        F64(f64) = sys::godot_variant_type_GODOT_VARIANT_TYPE_REAL,
        GodotString(GodotString) = sys::godot_variant_type_GODOT_VARIANT_TYPE_STRING,
        Vector2(Vector2) = sys::godot_variant_type_GODOT_VARIANT_TYPE_VECTOR2,
        Rect2(Rect2) = sys::godot_variant_type_GODOT_VARIANT_TYPE_RECT2,
        Vector3(Vector3) = sys::godot_variant_type_GODOT_VARIANT_TYPE_VECTOR3,
        Transform2D(Transform2D) = sys::godot_variant_type_GODOT_VARIANT_TYPE_TRANSFORM2D,
        Plane(Plane) = sys::godot_variant_type_GODOT_VARIANT_TYPE_PLANE,
        Quat(Quat) = sys::godot_variant_type_GODOT_VARIANT_TYPE_QUAT,
        Aabb(Aabb) = sys::godot_variant_type_GODOT_VARIANT_TYPE_AABB,
        Basis(Basis) = sys::godot_variant_type_GODOT_VARIANT_TYPE_BASIS,
        Transform(Transform) = sys::godot_variant_type_GODOT_VARIANT_TYPE_TRANSFORM,
        Color(Color) = sys::godot_variant_type_GODOT_VARIANT_TYPE_COLOR,
        NodePath(NodePath) = sys::godot_variant_type_GODOT_VARIANT_TYPE_NODE_PATH,
        Rid(Rid) = sys::godot_variant_type_GODOT_VARIANT_TYPE_RID,
        Object(Variant) = sys::godot_variant_type_GODOT_VARIANT_TYPE_OBJECT,
        Dictionary(Dictionary) = sys::godot_variant_type_GODOT_VARIANT_TYPE_DICTIONARY,
        VariantArray(VariantArray) = sys::godot_variant_type_GODOT_VARIANT_TYPE_ARRAY,
        ByteArray(ByteArray) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_BYTE_ARRAY,
        Int32Array(Int32Array) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_INT_ARRAY,
        Float32Array(Float32Array) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_REAL_ARRAY,
        StringArray(StringArray) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_STRING_ARRAY,
        Vector2Array(Vector2Array) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_VECTOR2_ARRAY,
        Vector3Array(Vector3Array) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_VECTOR3_ARRAY,
        ColorArray(ColorArray) = sys::godot_variant_type_GODOT_VARIANT_TYPE_POOL_COLOR_ARRAY,
    }
);

impl VariantType {
    #[doc(hidden)]
    #[inline]
    pub fn from_sys(v: sys::godot_variant_type) -> VariantType {
        unsafe { transmute(v as u32) }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallError {
    InvalidMethod =
        sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_ERROR_INVALID_METHOD as u32,
    InvalidArgument =
        sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_ERROR_INVALID_ARGUMENT as u32,
    TooManyArguments =
        sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_ERROR_TOO_MANY_ARGUMENTS as u32,
    TooFewArguments =
        sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_ERROR_TOO_FEW_ARGUMENTS as u32,
    InstanceIsNull =
        sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_ERROR_INSTANCE_IS_NULL as u32,
}

impl CallError {
    #[inline]
    fn from_sys(v: sys::godot_variant_call_error_error) -> Result<(), CallError> {
        if v == sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_OK {
            Ok(())
        } else {
            debug_assert!(
                (v as u32) <= sys::godot_variant_call_error_error_GODOT_CALL_ERROR_CALL_ERROR_INSTANCE_IS_NULL as u32,
                "Godot should have passed a known error",
            );

            Err(unsafe { transmute(v as u32) })
        }
    }
}

impl std::fmt::Display for CallError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CallError::*;
        match self {
            InvalidMethod => write!(f, "invalid method"),
            InvalidArgument => write!(f, "invalid argument"),
            TooManyArguments => write!(f, "too many arguments"),
            TooFewArguments => write!(f, "too few arguments"),
            InstanceIsNull => write!(f, "instance is null"),
        }
    }
}

impl std::error::Error for CallError {}

/// Godot variant operator kind.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VariantOperator {
    // Comparison
    Equal = sys::godot_variant_operator_GODOT_VARIANT_OP_EQUAL as u32,
    NotEqual = sys::godot_variant_operator_GODOT_VARIANT_OP_NOT_EQUAL as u32,
    Less = sys::godot_variant_operator_GODOT_VARIANT_OP_LESS as u32,
    LessEqual = sys::godot_variant_operator_GODOT_VARIANT_OP_LESS_EQUAL as u32,
    Greater = sys::godot_variant_operator_GODOT_VARIANT_OP_GREATER as u32,
    GreaterEqual = sys::godot_variant_operator_GODOT_VARIANT_OP_GREATER_EQUAL as u32,

    // Mathematic
    Add = sys::godot_variant_operator_GODOT_VARIANT_OP_ADD as u32,
    Subtract = sys::godot_variant_operator_GODOT_VARIANT_OP_SUBTRACT as u32,
    Multiply = sys::godot_variant_operator_GODOT_VARIANT_OP_MULTIPLY as u32,
    Divide = sys::godot_variant_operator_GODOT_VARIANT_OP_DIVIDE as u32,
    Negate = sys::godot_variant_operator_GODOT_VARIANT_OP_NEGATE as u32,
    Positive = sys::godot_variant_operator_GODOT_VARIANT_OP_POSITIVE as u32,
    Module = sys::godot_variant_operator_GODOT_VARIANT_OP_MODULE as u32,
    StringConcat = sys::godot_variant_operator_GODOT_VARIANT_OP_STRING_CONCAT as u32,

    // Bitwise
    ShiftLeft = sys::godot_variant_operator_GODOT_VARIANT_OP_SHIFT_LEFT as u32,
    ShiftRight = sys::godot_variant_operator_GODOT_VARIANT_OP_SHIFT_RIGHT as u32,
    BitAnd = sys::godot_variant_operator_GODOT_VARIANT_OP_BIT_AND as u32,
    BitOr = sys::godot_variant_operator_GODOT_VARIANT_OP_BIT_OR as u32,
    BitXor = sys::godot_variant_operator_GODOT_VARIANT_OP_BIT_XOR as u32,
    BitNegate = sys::godot_variant_operator_GODOT_VARIANT_OP_BIT_NEGATE as u32,

    // Logic
    And = sys::godot_variant_operator_GODOT_VARIANT_OP_AND as u32,
    Or = sys::godot_variant_operator_GODOT_VARIANT_OP_OR as u32,
    Xor = sys::godot_variant_operator_GODOT_VARIANT_OP_XOR as u32,
    Not = sys::godot_variant_operator_GODOT_VARIANT_OP_NOT as u32,

    // Containment
    In = sys::godot_variant_operator_GODOT_VARIANT_OP_IN as u32,
}

impl VariantOperator {
    const MAX: u32 = sys::godot_variant_operator_GODOT_VARIANT_OP_MAX as u32;

    #[doc(hidden)]
    #[inline]
    pub fn to_sys(self) -> sys::godot_variant_operator {
        self as u32 as sys::godot_variant_operator
    }

    #[doc(hidden)]
    #[inline]
    pub fn try_from_sys(op: sys::godot_variant_operator) -> Option<Self> {
        let op = op as u32;
        if op >= Self::MAX {
            return None;
        }

        // SAFETY: Max value is checked, and Self is repr(u32)
        unsafe { std::mem::transmute(op) }
    }
}

/// Error indicating that an operator result is invalid.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct InvalidOp;

//fn to_godot_varianty_type(v: VariantType) -> sys::godot_variant_type {
//    unsafe { transmute(v) }
//}

// These aliases are just here so the type name matches the VariantType's variant names
// to make writing macros easier.
type F64 = f64;
type I64 = i64;
type Bool = bool;

impl Variant {
    variant_constructors!(
        /// Creates a `Variant` wrapping a `Vector2`.
        #[inline]
        pub fn from_vector2(&Vector2) -> Self;
        /// Creates a `Variant` wrapping a `Vector3`.
        #[inline]
        pub fn from_vector3(&Vector3) -> Self;
        /// Creates a `Variant` wrapping a `Quat`.
        #[inline]
        pub fn from_quat(&Quat) -> Self;
        /// Creates a `Variant` wrapping a `Plane`.
        #[inline]
        pub fn from_plane(&Plane) -> Self;
        /// Creates a `Variant` wrapping a `Rect2`.
        #[inline]
        pub fn from_rect2(&Rect2) -> Self;
        /// Creates a `Variant` wrapping a `Transform`.
        #[inline]
        pub fn from_transform(&Transform) -> Self;
        /// Creates a `Variant` wrapping a `Transform2D`.
        #[inline]
        pub fn from_transform2d(&Transform2D) -> Self;
        /// Creates a `Variant` wrapping a `Basis`.
        #[inline]
        pub fn from_basis(&Basis) -> Self;
        /// Creates a `Variant` wrapping a `Color`.
        #[inline]
        pub fn from_color(&Color) -> Self;
        /// Creates a `Variant` wrapping an `Aabb`.
        #[inline]
        pub fn from_aabb(&Aabb) -> Self;
        /// Creates a `Variant` wrapping an `Rid`.
        #[inline]
        pub fn from_rid(&Rid) -> Self;
        /// Creates a `Variant` wrapping a `NodePath`.
        #[inline]
        pub fn from_node_path(&NodePath) -> Self;
        /// Creates a `Variant` wrapping a `GodotString`.
        #[inline]
        pub fn from_godot_string(&GodotString) -> Self;
        /// Creates an `Variant` wrapping an array of variants.
        #[inline]
        pub fn from_array(&VariantArray<Shared>) -> Self;
        /// Creates a `Variant` wrapping a byte array.
        #[inline]
        pub fn from_byte_array(&ByteArray) -> Self;
        /// Creates a `Variant` wrapping an array of 32bit signed integers.
        #[inline]
        pub fn from_int32_array(&Int32Array) -> Self;
        /// Creates a `Variant` wrapping an array of 32bit floats.
        #[inline]
        pub fn from_float32_array(&Float32Array) -> Self;
        /// Creates a `Variant` wrapping an array of godot strings.
        #[inline]
        pub fn from_string_array(&StringArray) -> Self;
        /// Creates a `Variant` wrapping an array of 2d vectors.
        #[inline]
        pub fn from_vector2_array(&Vector2Array) -> Self;
        /// Creates a `Variant` wrapping an array of 3d vectors.
        #[inline]
        pub fn from_vector3_array(&Vector3Array) -> Self;
        /// Creates a `Variant` wrapping an array of colors.
        #[inline]
        pub fn from_color_array(&ColorArray) -> Self;
        /// Creates a `Variant` wrapping a dictionary.
        #[inline]
        pub fn from_dictionary(&Dictionary<Shared>) -> Self;
    );

    /// Creates an empty `Variant`.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_nil)(&mut dest);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a string.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<S>(s: S) -> Variant
    where
        S: AsRef<str>,
    {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            let val = s.as_ref();
            let mut godot_s =
                (api.godot_string_chars_to_utf8_with_len)(val.as_ptr() as *const _, val.len() as _);
            (api.godot_variant_new_string)(&mut dest, &godot_s);
            (api.godot_string_destroy)(&mut godot_s);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a Godot object.
    #[inline]
    pub fn from_object<R>(val: R) -> Variant
    where
        R: AsVariant,
    {
        unsafe { R::to_arg_variant(&val) }
    }

    /// Creats a `Variant` from a raw object pointer.
    ///
    /// # Safety
    ///
    /// The object pointer must be a valid pointer to a godot object.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_object_ptr(val: *mut sys::godot_object) -> Variant {
        let api = get_api();
        let mut dest = sys::godot_variant::default();
        (api.godot_variant_new_object)(&mut dest, val);
        Variant(dest)
    }

    /// Creates a `Variant` wrapping a signed integer value.
    #[inline]
    pub fn from_i64(v: i64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_int)(&mut dest, v);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping an unsigned integer value.
    #[inline]
    pub fn from_u64(v: u64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_uint)(&mut dest, v);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a double-precision float value.
    #[inline]
    pub fn from_f64(v: f64) -> Variant {
        unsafe {
            let api = get_api();
            let mut ret = sys::godot_variant::default();
            (api.godot_variant_new_real)(&mut ret, v);
            Variant(ret)
        }
    }

    /// Creates a `Variant` wrapping an boolean.
    #[inline]
    pub fn from_bool(v: bool) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_bool)(&mut dest, v);
            Variant(dest)
        }
    }

    #[inline]
    fn try_as_sys_of_type(
        &self,
        expected: VariantType,
    ) -> Result<&sys::godot_variant, FromVariantError> {
        let variant_type = self.get_type();
        if variant_type != expected {
            return Err(FromVariantError::InvalidVariantType {
                expected,
                variant_type,
            });
        }
        Ok(&self.0)
    }

    variant_to_type_transmute!(
        /// Do a best effort to create a `Vector2` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_vector2(&self) -> Vector2 : godot_variant_as_vector2;
        /// Returns `Some(Vector2)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_vector2(&self) -> Option<Vector2>;

        /// Do a best effort to create a `Vector3` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_vector3(&self) -> Vector3 : godot_variant_as_vector3;
        /// Returns `Some(Vector3)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_vector3(&self) -> Option<Vector3>;

        /// Do a best effort to create a `Quat` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_quat(&self) -> Quat : godot_variant_as_quat;
        /// Returns `Some(Quat)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_quat(&self) -> Option<Quat>;

        /// Do a best effort to create a `Rect2` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_rect2(&self) -> Rect2 : godot_variant_as_rect2;
        /// Returns `Some(Rect2)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_rect2(&self) -> Option<Rect2>;

        /// Do a best effort to create a `Transform2D` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_transform2d(&self) -> Transform2D : godot_variant_as_transform2d;
        /// Returns `Some(Transform2D)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_transform2d(&self) -> Option<Transform2D>;

        /// Do a best effort to create a `f64` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_f64(&self) -> F64 : godot_variant_as_real;
        /// Returns `Some(f64)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_f64(&self) -> Option<F64>;

        /// Do a best effort to create an `i64` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_i64(&self) -> I64 : godot_variant_as_int;
        /// Returns `Some(i64)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_i64(&self) -> Option<I64>;

        /// Do a best effort to create a `bool` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_bool(&self) -> Bool : godot_variant_as_bool;
        /// Returns `Some(bool)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_bool(&self) -> Option<Bool>;
    );

    /// Do a best effort to create a `u64` out of the variant, possibly returning a default value.
    #[inline]
    pub fn to_u64(&self) -> u64 {
        unsafe {
            let api = get_api();
            (api.godot_variant_as_uint)(&self.0)
        }
    }

    /// Returns `Some(u64)` if this variant is one, `None` otherwise.
    #[inline]
    pub fn try_to_u64(&self) -> Option<u64> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(&self.0)
                == sys::godot_variant_type_GODOT_VARIANT_TYPE_INT
            {
                Some((api.godot_variant_as_uint)(&self.0))
            } else {
                None
            }
        }
    }

    variant_to_type_from_sys!(
        /// Do a best effort to create a `Plane` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_plane(&self) -> Plane : godot_variant_as_plane;
        /// Returns `Some(Plane)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_plane(&self) -> Option<Plane>;

        /// Do a best effort to create a `Transform` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_transform(&self) -> Transform : godot_variant_as_transform;
        /// Returns `Some(Transform)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_transform(&self) -> Option<Transform>;

        /// Do a best effort to create a `Color` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_color(&self) -> Color : godot_variant_as_color;
        /// Returns `Some(Color)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_color(&self) -> Option<Color>;

        /// Do a best effort to create a `Basis` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_basis(&self) -> Basis : godot_variant_as_basis;
        /// Returns `Some(Basis)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_basis(&self) -> Option<Basis>;

        /// Do a best effort to create an `Aabb` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_aabb(&self) -> Aabb : godot_variant_as_aabb;
        /// Returns `Some(Aabb)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_aabb(&self) -> Option<Aabb>;

        /// Do a best effort to create a `NodePath` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_node_path(&self) -> NodePath : godot_variant_as_node_path;
        /// Returns `Some(NodePath)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_node_path(&self) -> Option<NodePath>;

        /// Do a best effort to create a `GodotString` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_godot_string(&self) -> GodotString : godot_variant_as_string;
        /// Returns `Some(GodotString)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_godot_string(&self) -> Option<GodotString>;

        /// Do a best effort to create a `Rid` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_rid(&self) -> Rid : godot_variant_as_rid;
        /// Returns `Some(Rid)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_rid(&self) -> Option<Rid>;

        /// Do a best effort to create a `VariantArray` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_array(&self) -> VariantArray<Shared> : godot_variant_as_array;
        /// Returns `Some(VariantArray)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_array(&self) -> Option<VariantArray<Shared>>;

        /// Do a best effort to create a `ByteArray` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_byte_array(&self) -> ByteArray : godot_variant_as_pool_byte_array;
        /// Returns `Some(ByteArray)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_byte_array(&self) -> Option<ByteArray>;

        /// Do a best effort to create an `Int32Array` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_int32_array(&self) -> Int32Array : godot_variant_as_pool_int_array;
        /// Returns `Some(Int32Array)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_int32_array(&self) -> Option<Int32Array>;

        /// Do a best effort to create a `Float32Array` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_float32_array(&self) -> Float32Array : godot_variant_as_pool_real_array;
        /// Returns `Some(Float32Array)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_float32_array(&self) -> Option<Float32Array>;

        /// Do a best effort to create a `StringArray` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_string_array(&self) -> StringArray : godot_variant_as_pool_string_array;
        /// Returns `Some(StringArray)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_string_array(&self) -> Option<StringArray>;

        /// Do a best effort to create a `Vector2Array` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_vector2_array(&self) -> Vector2Array : godot_variant_as_pool_vector2_array;
        /// Returns `Some(Vector2Array)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_vector2_array(&self) -> Option<Vector2Array>;

        /// Do a best effort to create a `Vector3Array` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_vector3_array(&self) -> Vector3Array : godot_variant_as_pool_vector3_array;
        /// Returns `Some(Vector3Array)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_vector3_array(&self) -> Option<Vector3Array>;

        /// Do a best effort to create a `ColorArray` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_color_array(&self) -> ColorArray : godot_variant_as_pool_color_array;
        /// Returns `Some(ColorArray)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_color_array(&self) -> Option<ColorArray>;

        /// Do a best effort to create a `Dictionary` out of the variant, possibly returning a default value.
        #[inline]
        pub fn to_dictionary(&self) -> Dictionary<Shared> : godot_variant_as_dictionary;
        /// Returns `Some(Dictionary)` if this variant is one, `None` otherwise.
        #[inline]
        pub fn try_to_dictionary(&self) -> Option<Dictionary<Shared>>;
    );

    #[inline]
    pub fn try_to_object<T>(&self) -> Option<Ref<T, Shared>>
    where
        T: GodotObject,
    {
        self.try_to_object_with_error::<T>().ok()
    }

    #[inline]
    pub fn try_to_object_with_error<T>(&self) -> Result<Ref<T, Shared>, FromVariantError>
    where
        T: GodotObject,
    {
        unsafe {
            let api = get_api();
            let obj = self.try_as_sys_of_type(VariantType::Object)?;
            let obj = ptr::NonNull::new((api.godot_variant_as_object)(obj))
                .ok_or(FromVariantError::InvalidNil)?;
            let obj =
                object::RawObject::<ManuallyManagedClassPlaceholder>::from_sys_ref_unchecked(obj);
            let obj = obj
                .cast::<T>()
                .ok_or_else(|| FromVariantError::CannotCast {
                    class: obj.class_name(),
                    to: T::class_name(),
                })?;

            Ok(Ref::from_sys(obj.sys()))
        }
    }

    #[inline]
    pub fn try_to_string(&self) -> Option<String> {
        self.try_to_godot_string().map(|s| s.to_string())
    }

    /// Returns this variant's type.
    #[inline]
    pub fn get_type(&self) -> VariantType {
        unsafe { VariantType::from_sys((get_api().godot_variant_get_type)(&self.0)) }
    }

    /// Converts this variant to a primitive value depending on its type.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let variant = 42.to_variant();
    /// let number_as_float = match variant.dispatch() {
    ///     VariantDispatch::I64(i) => i as f64,
    ///     VariantDispatch::F64(f) => f,
    ///     _ => panic!("not a number"),
    /// };
    /// approx::assert_relative_eq!(42.0, number_as_float);
    /// ```
    #[inline]
    pub fn dispatch(&self) -> VariantDispatch {
        self.into()
    }

    /// Returns true if this is an empty variant.
    #[inline]
    pub fn is_nil(&self) -> bool {
        self.get_type() == VariantType::Nil
    }

    #[inline]
    pub fn has_method(&self, method: impl Into<GodotString>) -> bool {
        let method = method.into();
        unsafe { (get_api().godot_variant_has_method)(&self.0, &method.0) }
    }

    #[inline]
    pub fn call(
        &mut self,
        method: impl Into<GodotString>,
        args: &[Variant],
    ) -> Result<Variant, CallError> {
        let method = method.into();
        unsafe {
            let api = get_api();
            let mut err = sys::godot_variant_call_error::default();
            let mut arg_refs = args.iter().map(Variant::sys).collect::<Vec<_>>();
            let variant = (api.godot_variant_call)(
                &mut self.0,
                &method.0,
                arg_refs.as_mut_ptr(),
                args.len() as i32,
                &mut err,
            );

            CallError::from_sys(err.error).map(|_| Variant::from_sys(variant))
        }
    }

    /// Evaluates a variant operator on `self` and `rhs` and returns the result on success.
    ///
    /// # Errors
    ///
    /// Returns `Err(InvalidOp)` if the result is not valid.
    #[inline]
    pub fn evaluate(&self, op: VariantOperator, rhs: &Self) -> Result<Variant, InvalidOp> {
        unsafe {
            let api = get_api();
            let mut ret = Variant::new();
            let mut valid = false;

            (api.godot_variant_evaluate)(
                op.to_sys(),
                self.sys(),
                rhs.sys(),
                ret.sys_mut(),
                &mut valid,
            );

            if valid {
                Ok(ret)
            } else {
                Err(InvalidOp)
            }
        }
    }

    /// Get a reference to a `godot-rust` Variant from a raw sys::pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid pointer to a `sys::godot_variant`.
    #[inline]
    pub(crate) unsafe fn cast_ref<'l>(ptr: *const sys::godot_variant) -> &'l Variant {
        &*(ptr as *const variant::Variant)
    }

    #[inline]
    pub(crate) fn cast_mut_ref<'l>(ptr: *mut sys::godot_variant) -> &'l mut Variant {
        unsafe { &mut *(ptr as *mut variant::Variant) }
    }

    /// Returns the internal ffi representation of the variant and consumes
    /// the rust object without running the destructor.
    ///
    /// This should be only used when certain that the receiving side is
    /// responsible for running the destructor for the object, otherwise
    /// it is leaked.
    #[inline]
    pub fn forget(self) -> sys::godot_variant {
        let v = self.0;
        forget(self);
        v
    }

    // Returns a copy of the internal ffi representation of the variant.
    //
    // The variant remains owned by the rust wrapper and the receiver of
    // the ffi representation should not run its destructor.
    #[doc(hidden)]
    #[inline]
    pub fn to_sys(&self) -> sys::godot_variant {
        self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_variant {
        &self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut sys::godot_variant {
        &mut self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_variant) -> Self {
        Variant(sys)
    }
}

impl_basic_traits_as_sys!(
    for Variant as godot_variant {
        Drop => godot_variant_destroy;
        Clone => godot_variant_new_copy;
        PartialEq => godot_variant_operator_equal;
    }
);

impl Eq for Variant {}

impl ToString for Variant {
    #[inline]
    fn to_string(&self) -> String {
        self.to_godot_string().to_string()
    }
}

impl Default for Variant {
    #[inline]
    fn default() -> Self {
        Variant::new()
    }
}

impl fmt::Debug for Variant {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}({})", self.get_type(), self.to_string())
    }
}

godot_test!(
    test_variant_nil {
        let nil = Variant::new();
        assert_eq!(nil.get_type(), VariantType::Nil);
        assert!(nil.is_nil());

        assert!(nil.try_to_array().is_none());
        assert!(nil.try_to_rid().is_none());
        assert!(nil.try_to_i64().is_none());
        assert!(nil.try_to_bool().is_none());
        assert!(nil.try_to_aabb().is_none());
        assert!(nil.try_to_vector2().is_none());
        assert!(nil.try_to_basis().is_none());

        assert!(!nil.has_method("foo"));

        let clone = nil.clone();
        assert!(clone == nil);
    }

    test_variant_i64 {
        let v_42 = Variant::from_i64(42);
        assert_eq!(v_42.get_type(), VariantType::I64);

        assert!(!v_42.is_nil());
        assert_eq!(v_42.try_to_i64(), Some(42));
        assert!(v_42.try_to_f64().is_none());
        assert!(v_42.try_to_array().is_none());

        let v_m1 = Variant::from_i64(-1);
        assert_eq!(v_m1.get_type(), VariantType::I64);

        assert!(!v_m1.is_nil());
        assert_eq!(v_m1.try_to_i64(), Some(-1));
        assert!(v_m1.try_to_f64().is_none());
        assert!(v_m1.try_to_array().is_none());
    }

    test_variant_bool {
        let v_true = Variant::from_bool(true);
        assert_eq!(v_true.get_type(), VariantType::Bool);

        assert!(!v_true.is_nil());
        assert_eq!(v_true.try_to_bool(), Some(true));
        assert!(v_true.try_to_f64().is_none());
        assert!(v_true.try_to_array().is_none());

        let v_false = Variant::from_bool(false);
        assert_eq!(v_false.get_type(), VariantType::Bool);

        assert!(!v_false.is_nil());
        assert_eq!(v_false.try_to_bool(), Some(false));
        assert!(v_false.try_to_f64().is_none());
        assert!(v_false.try_to_array().is_none());

    }
);

/// Types that can be converted to a `Variant`.
///
/// ## Wrappers and collections
///
/// Implementations are provided for a few common Rust wrappers and collections:
///
/// - `Option<T>` is unwrapped to inner value, or `Nil` if `None`
/// - `Result<T, E>` is represented as an externally tagged `Dictionary` (see below).
/// - `PhantomData<T>` is represented as `Nil`.
/// - `&[T]` and `Vec<T>` are represented as `VariantArray`s. `FromVariant` is only implemented
/// for `Vec<T>`.
///
/// ## Deriving `ToVariant`
///
/// The derive macro does the following mapping between Rust structures and Godot types:
///
/// - `Newtype(inner)` is unwrapped to `inner`
/// - `Tuple(a, b, c)` is represented as a `VariantArray` (`[a, b, c]`)
/// - `Struct { a, b, c }` is represented as a `Dictionary` (`{ "a": a, "b": b, "c": c }`)
/// - `Unit` is represented as an empty `Dictionary` (`{}`)
/// - `Enum::Variant(a, b, c)` is represented as an externally tagged `Dictionary`
///   (`{ "Variant": [a, b, c] }`)
///
/// Behavior of the derive macros can be customized using attributes:
///
/// ### Field attributes
///
/// - `#[variant(to_variant_with = "path::to::func")]`
///
/// Use the given function to convert the field to `Variant`. The function's signature is
/// expected to be `fn(&T) -> Variant`, although it can be generic over `T`.
///
/// - `#[variant(from_variant_with = "path::to::func")]`
///
/// Use the given function to convert from a `Variant`. The function's signature is
/// expected to be `fn(&Variant) -> Result<T, FromVariantError>`, although it can be generic
/// over `T`.
///
/// - `#[variant(with = "path::to::mod")]`
///
/// Convenience attribute that sets `to_variant_with` to `path::to::mod::to_variant` and
/// `from_variant_with` to `path::to::mod::from_variant`.
///
/// - `#[variant(skip_to_variant)]`
///
/// Skip the field when converting to `Variant`.
///
/// - `#[variant(skip_from_variant)]`
///
/// Skip the field when converting from `Variant`. A default vale will be obtained using
/// `Default::default()`.
///
/// - `#[variant(skip)]`
///
/// Convenience attribute that sets `skip_to_variant` and `skip_from_variant`.
pub trait ToVariant {
    fn to_variant(&self) -> Variant;
}

/// Types that can only be safely converted to a `Variant` as owned values. Such types cannot
/// implement `ToVariant` in general, but can still be passed to API methods as arguments, or
/// used as return values. Notably, this includes `Unique` arrays, dictionaries, and references
/// to Godot objects and instances.
///
/// This has a blanket implementation for all types that have `ToVariant`. As such, users
/// should only derive or implement `OwnedToVariant` when `ToVariant` is not applicable.
///
/// ## Deriving `OwnedToVariant`
///
/// The derive macro behaves the same as `ToVariant`. See the documentation for the latter for
/// a detailed explanation.
pub trait OwnedToVariant {
    fn owned_to_variant(self) -> Variant;
}

/// Trait for types whose `ToVariant` implementations preserve equivalence.
///
/// This means that for all values `a` and `b`, `a == b` is equivalent to
/// `a.to_variant() == b.to_variant()`. Most of the time, this means that `to_variant` must
/// return a "value" type, such as a primitive `i32`, a `GodotString`, or a `TypedArray`.
///
/// This is mostly useful as a bound for `Dictionary` keys, where the difference between Rust's
/// structural equality and Godot's referential equality semantics can lead to surprising
/// behaviors.
///
/// This property cannot be checked by the compiler, so `ToVariantEq` has no extra methods.
///
/// ## Implementing `ToVariantEq`
///
/// The `ToVariantEq` trait is not derivable, because most derived implementations of
/// `ToVariant` don't satisfy the requirements. If you are sure that your type satisfies the
/// trait, specify that your type implements it with an empty `impl`:
///
/// ```ignore
/// #[derive(Eq, PartialEq, ToVariant)]
/// struct MyTypedInt(i32);
///
/// impl ToVariantEq for MyTypedInt {}
/// ```
pub trait ToVariantEq: Eq {}

/// Types that can be converted from a `Variant`.
///
/// ## `Option<T>` and `MaybeNot<T>`
///
/// `Option<T>` requires the Variant to be `T` or `Nil`, in that order. For looser semantics,
/// use `MaybeNot<T>`, which will catch all variant values that are not `T` as well.
///
/// ## `Vec<T>`
///
/// The `FromVariant` implementation for `Vec<T>` only allow homogeneous arrays. If you want to
/// manually handle potentially heterogeneous values e.g. for error reporting, use `VariantArray`
/// directly or compose with an appropriate wrapper: `Vec<Option<T>>` or `Vec<MaybeNot<T>>`.
///
/// ## Deriving `FromVariant`
///
/// The derive macro provides implementation consistent with derived `ToVariant`. See `ToVariant`
/// for detailed documentation.
pub trait FromVariant: Sized {
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
/// Error type returned by `FromVariant::from_variant`.
pub enum FromVariantError {
    /// An unspecified error.
    Unspecified,
    /// A custom error message.
    Custom(String),
    /// Null value given for a non-nullable type, with no further information given.
    InvalidNil,
    /// Variant type is different from the expected one.
    InvalidVariantType {
        variant_type: VariantType,
        expected: VariantType,
    },
    /// Cannot cast the object to the given Godot class.
    CannotCast { class: String, to: &'static str },
    /// Length of the collection is different from the expected one.
    InvalidLength { len: usize, expected: usize },
    /// Invalid enum representation.
    InvalidEnumRepr {
        expected: VariantEnumRepr,
        error: Box<FromVariantError>,
    },
    /// Invalid struct representation.
    InvalidStructRepr {
        expected: VariantStructRepr,
        error: Box<FromVariantError>,
    },

    /// Error indicating that the implementation encountered an enum variant that does not exist
    /// at compile time.
    ///
    /// For example, trying to create a `Result<T, E>` from `{ "Foo": "Bar" }` will result in this
    /// error, since `Foo` is not a valid variant of `Result`.
    UnknownEnumVariant {
        /// Name of the unknown variant
        variant: String,
        /// Names of all expected variants known at compile time
        expected: &'static [&'static str],
    },

    /// Error indicating that the implementation encountered a known enum variant, but the value
    /// is invalid for the definition.
    ///
    /// This could result from multiple underlying reasons, detailed in the `error` field:
    ///
    /// - Missing fields.
    /// - Unexpected representation, e.g. `{ "0": "foo", "1": "bar" }` for a tuple.
    /// - Error in a nested field.
    InvalidEnumVariant {
        variant: &'static str,
        error: Box<FromVariantError>,
    },

    /// Given object is not an instance of the expected NativeClass.
    InvalidInstance { expected: &'static str },
    /// Collection contains an invalid field.
    InvalidField {
        field_name: &'static str,
        error: Box<FromVariantError>,
    },
    /// Collection contains an invalid item.
    InvalidItem {
        index: usize,
        error: Box<FromVariantError>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum VariantEnumRepr {
    ExternallyTagged,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum VariantStructRepr {
    Unit,
    Tuple,
    Struct,
}

impl FromVariantError {
    /// Returns a `FromVariantError` with a custom message.
    #[inline]
    pub fn custom<T: fmt::Display>(message: T) -> Self {
        FromVariantError::Custom(format!("{}", message))
    }
}

impl fmt::Display for FromVariantError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FromVariantError as E;

        match self {
            E::Unspecified => write!(f, "unspecified error"),
            E::Custom(s) => write!(f, "{}", s),
            E::InvalidNil => write!(f, "expected non-nullable type, got null"),
            E::InvalidVariantType {
                variant_type,
                expected,
            } => write!(
                f,
                "invalid variant type: expected {:?}, got {:?}",
                expected, variant_type
            ),
            E::CannotCast { class, to } => {
                write!(f, "cannot cast object of class {} to {}", class, to)
            }
            E::InvalidLength { len, expected } => {
                write!(f, "expected collection of length {}, got {}", expected, len)
            }
            E::InvalidEnumRepr { expected, error } => write!(
                f,
                "invalid enum representation: expected {:?}, {}",
                expected, error
            ),
            E::InvalidStructRepr { expected, error } => write!(
                f,
                "invalid struct representation: expected {:?}, {}",
                expected, error
            ),
            E::UnknownEnumVariant { variant, expected } => {
                write!(
                    f,
                    "unknown enum variant {}, expected variants are: ",
                    variant
                )?;
                let mut first = true;
                for v in *expected {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                Ok(())
            }
            E::InvalidEnumVariant { variant, error } => {
                write!(f, "invalid value for variant {}: {}", variant, error)
            }
            E::InvalidInstance { expected } => {
                write!(f, "object is not an instance of `NativeClass` {}", expected)
            }
            E::InvalidField { field_name, error } => {
                write!(f, "invalid value for field {}", field_name)?;

                let mut next_error = error.as_ref();
                loop {
                    match next_error {
                        E::InvalidField { field_name, error } => {
                            write!(f, ".{}", field_name)?;
                            next_error = error.as_ref();
                        }
                        E::InvalidItem { index, error } => {
                            write!(f, "[{}]", index)?;
                            next_error = error.as_ref();
                        }
                        _ => {
                            write!(f, ": {}", next_error)?;
                            return Ok(());
                        }
                    }
                }
            }
            E::InvalidItem { index, error } => {
                write!(f, "invalid value for item at index {}: {}", index, error)
            }
        }
    }
}

impl std::error::Error for FromVariantError {}

impl<T: ToVariant> OwnedToVariant for T {
    #[inline]
    fn owned_to_variant(self) -> Variant {
        self.to_variant()
    }
}

impl ToVariant for () {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::new()
    }
}
impl ToVariantEq for () {}

impl FromVariant for () {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        variant.try_as_sys_of_type(VariantType::Nil).map(|_| ())
    }
}

impl<'a, T> ToVariant for &'a T
where
    T: ToVariant + ?Sized,
{
    #[inline]
    fn to_variant(&self) -> Variant {
        T::to_variant(*self)
    }
}
impl<'a, T> ToVariantEq for &'a T where T: ToVariantEq + ?Sized {}

impl ToVariantEq for Variant {}

impl<'a, T> ToVariant for &'a mut T
where
    T: ToVariant + ?Sized,
{
    #[inline]
    fn to_variant(&self) -> Variant {
        T::to_variant(*self)
    }
}
impl<'a, T> ToVariantEq for &'a mut T where T: ToVariantEq + ?Sized {}

impl<T: GodotObject> ToVariant for Ref<T, Shared> {
    #[inline]
    fn to_variant(&self) -> Variant {
        unsafe { Variant::from_object_ptr(self.as_ptr()) }
    }
}

impl<T: GodotObject> OwnedToVariant for Ref<T, Unique> {
    #[inline]
    fn owned_to_variant(self) -> Variant {
        unsafe { Variant::from_object_ptr(self.as_ptr()) }
    }
}

impl<'a, T: GodotObject> ToVariant for TRef<'a, T, Shared> {
    #[inline]
    fn to_variant(&self) -> Variant {
        unsafe { Variant::from_object_ptr(self.as_ptr()) }
    }
}

impl<T: GodotObject> FromVariant for Ref<T, Shared> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        variant.try_to_object_with_error::<T>()
    }
}

macro_rules! from_variant_direct {
    (
        $(
            impl FromVariant for $TryType:ident : VariantType :: $VarType:ident => $try_gd_method:ident;
        )*
    ) => (
        $(
            impl FromVariant for $TryType {
                #[inline]
                fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
                    variant.try_as_sys_of_type(VariantType::$VarType)
                        .map(|v| unsafe { (get_api().$try_gd_method)(v) })
                }
            }
        )*
    );
}

from_variant_direct!(
    impl FromVariant for f64 : VariantType::F64 => godot_variant_as_real;
    impl FromVariant for i64 : VariantType::I64 => godot_variant_as_int;
    impl FromVariant for u64 : VariantType::I64 => godot_variant_as_uint;
    impl FromVariant for bool : VariantType::Bool => godot_variant_as_bool;
);

impl ToVariant for i64 {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::from_i64(*self)
    }
}
impl ToVariantEq for i64 {}

impl ToVariant for u64 {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::from_u64(*self)
    }
}
impl ToVariantEq for u64 {}

impl ToVariant for f64 {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::from_f64(*self)
    }
}

macro_rules! impl_to_variant_for_num {
    (
        $($ty:ty : $src_ty:ty)*
    ) => {
        $(
            impl ToVariant for $ty {
                #[inline]
                fn to_variant(&self) -> Variant {
                    ((*self) as $src_ty).to_variant()
                }
            }

            impl FromVariant for $ty {
                #[inline]
                fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
                    <$src_ty>::from_variant(variant).map(|i| i as Self)
                }
            }
        )*
    };
}

impl_to_variant_for_num!(
    i8: i64
    i16: i64
    i32: i64
    isize: i64
    u8: u64
    u16: u64
    u32: u64
    usize: u64
    f32: f64
);

impl ToVariantEq for i8 {}
impl ToVariantEq for i16 {}
impl ToVariantEq for i32 {}
impl ToVariantEq for isize {}
impl ToVariantEq for u8 {}
impl ToVariantEq for u16 {}
impl ToVariantEq for u32 {}
impl ToVariantEq for usize {}

macro_rules! to_variant_transmute {
    (
        $(impl ToVariant for $ty:ident: $ctor:ident;)*
    ) => {
        $(
            impl ToVariant for $ty {
                #[inline]
                fn to_variant(&self) -> Variant {
                    unsafe {
                        let api = get_api();
                        let mut dest = sys::godot_variant::default();
                        #[allow(clippy::useless_transmute)]
                        (api.$ctor)(&mut dest, transmute(self));
                        Variant::from_sys(dest)
                    }
                }
            }
        )*
    }
}

to_variant_transmute! {
    impl ToVariant for Vector2 : godot_variant_new_vector2;
    impl ToVariant for Vector3 : godot_variant_new_vector3;
    impl ToVariant for Quat : godot_variant_new_quat;
    impl ToVariant for Rect2 : godot_variant_new_rect2;
    impl ToVariant for Transform2D : godot_variant_new_transform2d;
}

macro_rules! to_variant_as_sys {
    (
        $(impl ToVariant for $ty:ty: $ctor:ident;)*
    ) => {
        $(
            impl ToVariant for $ty {
                #[inline]
                fn to_variant(&self) -> Variant {
                    unsafe {
                        let api = get_api();
                        let mut dest = sys::godot_variant::default();
                        (api.$ctor)(&mut dest, self.sys());
                        Variant::from_sys(dest)
                    }
                }
            }
        )*
    }
}

to_variant_as_sys! {
    impl ToVariant for Plane : godot_variant_new_plane;
    impl ToVariant for Transform : godot_variant_new_transform;
    impl ToVariant for Basis : godot_variant_new_basis;
    impl ToVariant for Color : godot_variant_new_color;
    impl ToVariant for Aabb : godot_variant_new_aabb;
    impl ToVariant for Rid : godot_variant_new_rid;
    impl ToVariant for NodePath : godot_variant_new_node_path;
    impl ToVariant for GodotString : godot_variant_new_string;
    impl ToVariant for VariantArray<Shared> : godot_variant_new_array;
    impl ToVariant for Dictionary<Shared> : godot_variant_new_dictionary;
}

impl ToVariantEq for Rid {}
impl ToVariantEq for NodePath {}
impl ToVariantEq for GodotString {}

impl OwnedToVariant for Dictionary<Unique> {
    #[inline]
    fn owned_to_variant(self) -> Variant {
        self.into_shared().to_variant()
    }
}

impl OwnedToVariant for VariantArray<Unique> {
    #[inline]
    fn owned_to_variant(self) -> Variant {
        self.into_shared().to_variant()
    }
}

macro_rules! from_variant_transmute {
    (
        $(
            impl FromVariant for $TryType:ident : $try_gd_method:ident;
        )*
    ) => (
        $(
            impl FromVariant for $TryType {
                #[inline]
                fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
                    unsafe {
                        variant.try_as_sys_of_type(VariantType::$TryType)
                            .map(|v| (get_api().$try_gd_method)(v))
                            .map(|v| transmute(v))
                    }
                }
            }
        )*
    );
}

from_variant_transmute!(
    impl FromVariant for Vector2 : godot_variant_as_vector2;
    impl FromVariant for Vector3 : godot_variant_as_vector3;
    impl FromVariant for Quat : godot_variant_as_quat;
    impl FromVariant for Rect2 : godot_variant_as_rect2;
    impl FromVariant for Transform2D : godot_variant_as_transform2d;
);

macro_rules! from_variant_from_sys {
    (
        $(
            impl FromVariant for $TryType:ty as $EnumVar:ident : $try_gd_method:ident;
        )*
    ) => (
        $(
            impl FromVariant for $TryType {
                #[inline]
                fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
                    unsafe {
                        variant.try_as_sys_of_type(VariantType::$EnumVar)
                            .map(|v| (get_api().$try_gd_method)(v))
                            .map(<$TryType>::from_sys)
                    }
                }
            }
        )*
    );
}

from_variant_from_sys!(
    impl FromVariant for Plane as Plane : godot_variant_as_plane;
    impl FromVariant for Transform as Transform : godot_variant_as_transform;
    impl FromVariant for Basis as Basis : godot_variant_as_basis;
    impl FromVariant for Color as Color : godot_variant_as_color;
    impl FromVariant for Aabb as Aabb : godot_variant_as_aabb;
    impl FromVariant for NodePath as NodePath : godot_variant_as_node_path;
    impl FromVariant for GodotString as GodotString: godot_variant_as_string;
    impl FromVariant for Rid as Rid : godot_variant_as_rid;
    impl FromVariant for VariantArray<Shared> as VariantArray : godot_variant_as_array;
    impl FromVariant for Dictionary<Shared> as Dictionary : godot_variant_as_dictionary;
);

impl<T: crate::core_types::typed_array::Element> ToVariant for TypedArray<T> {
    #[inline]
    fn to_variant(&self) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (T::array_to_variant_fn(api))(&mut dest, self.sys());
            Variant::from_sys(dest)
        }
    }
}
impl<T: crate::core_types::typed_array::Element + Eq> ToVariantEq for TypedArray<T> {}

impl<T: crate::core_types::typed_array::Element> FromVariant for TypedArray<T> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        unsafe {
            variant
                .try_as_sys_of_type(VariantType::from_sys(T::SYS_VARIANT_TYPE))
                .map(|v| (T::array_from_variant_fn(get_api()))(v))
                .map(Self::from_sys)
        }
    }
}

impl ToVariant for str {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::from_str(self)
    }
}
impl ToVariantEq for str {}

impl ToVariant for String {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::from_str(&self)
    }
}
impl ToVariantEq for String {}

impl FromVariant for String {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        GodotString::from_variant(variant).map(|s| s.to_string())
    }
}

impl ToVariant for bool {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::from_bool(*self)
    }
}
impl ToVariantEq for bool {}

impl ToVariant for Variant {
    #[inline]
    fn to_variant(&self) -> Variant {
        self.clone()
    }
}

impl FromVariant for Variant {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        Ok(variant.clone())
    }
}

impl<T> ToVariant for std::marker::PhantomData<T> {
    #[inline]
    fn to_variant(&self) -> Variant {
        Variant::new()
    }
}
impl<T> ToVariantEq for std::marker::PhantomData<T> {}

impl<T> FromVariant for std::marker::PhantomData<T> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        variant
            .try_as_sys_of_type(VariantType::Nil)
            .map(|_| std::marker::PhantomData)
    }
}

impl<T: ToVariant> ToVariant for Option<T> {
    #[inline]
    fn to_variant(&self) -> Variant {
        match &self {
            Some(thing) => thing.to_variant(),
            None => Variant::new(),
        }
    }
}
impl<T: ToVariantEq> ToVariantEq for Option<T> {}

impl<T: FromVariant> FromVariant for Option<T> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        T::from_variant(variant).map(Some).or_else(
            |e| {
                if variant.is_nil() {
                    Ok(None)
                } else {
                    Err(e)
                }
            },
        )
    }
}

/// Wrapper type around a `FromVariant` result that may not be a success
#[derive(Clone, Debug)]
pub struct MaybeNot<T>(Result<T, Variant>);

impl<T: FromVariant> FromVariant for MaybeNot<T> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        Ok(MaybeNot(
            T::from_variant(variant).map_err(|_| variant.clone()),
        ))
    }
}

impl<T> MaybeNot<T> {
    #[inline]
    pub fn into_result(self) -> Result<T, Variant> {
        self.0
    }

    #[inline]
    pub fn as_ref(&self) -> Result<&T, &Variant> {
        self.0.as_ref()
    }

    #[inline]
    pub fn as_mut(&mut self) -> Result<&mut T, &mut Variant> {
        self.0.as_mut()
    }

    #[inline]
    pub fn cloned(&self) -> Result<T, Variant>
    where
        T: Clone,
    {
        self.0.clone()
    }

    #[inline]
    pub fn ok(self) -> Option<T> {
        self.0.ok()
    }
}

impl<T: ToVariant, E: ToVariant> ToVariant for Result<T, E> {
    #[inline]
    fn to_variant(&self) -> Variant {
        let dict = Dictionary::new();
        match &self {
            Ok(val) => dict.insert("Ok", val),
            Err(err) => dict.insert("Err", err),
        }
        dict.into_shared().to_variant()
    }
}

impl<T: FromVariant, E: FromVariant> FromVariant for Result<T, E> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        use FromVariantError as FVE;

        let dict = Dictionary::from_variant(variant).map_err(|err| FVE::InvalidEnumRepr {
            expected: VariantEnumRepr::ExternallyTagged,
            error: Box::new(err),
        })?;

        if dict.len() != 1 {
            return Err(FVE::InvalidEnumRepr {
                expected: VariantEnumRepr::ExternallyTagged,
                error: Box::new(FVE::InvalidLength {
                    expected: 1,
                    len: dict.len() as usize,
                }),
            });
        }

        let keys = dict.keys();
        let key_variant = &keys.get(0);
        let key = String::from_variant(key_variant).map_err(|err| FVE::InvalidEnumRepr {
            expected: VariantEnumRepr::ExternallyTagged,
            error: Box::new(err),
        })?;

        match key.as_str() {
            "Ok" => {
                let val = T::from_variant(&dict.get_or_nil(key_variant)).map_err(|err| {
                    FVE::InvalidEnumVariant {
                        variant: "Ok",
                        error: Box::new(err),
                    }
                })?;
                Ok(Ok(val))
            }
            "Err" => {
                let err = E::from_variant(&dict.get_or_nil(key_variant)).map_err(|err| {
                    FVE::InvalidEnumVariant {
                        variant: "Err",
                        error: Box::new(err),
                    }
                })?;
                Ok(Err(err))
            }
            variant => Err(FVE::UnknownEnumVariant {
                variant: variant.to_string(),
                expected: &["Ok", "Err"],
            }),
        }
    }
}

impl<T: ToVariant> ToVariant for &[T] {
    #[inline]
    fn to_variant(&self) -> Variant {
        let array = VariantArray::new();
        for val in self.iter() {
            // there is no real way to avoid CoW allocations right now, as ptrw isn't exposed
            array.push(&val.to_variant());
        }
        array.into_shared().to_variant()
    }
}

impl<T: ToVariant> ToVariant for Vec<T> {
    #[inline]
    fn to_variant(&self) -> Variant {
        self.as_slice().to_variant()
    }
}

impl<T: FromVariant> FromVariant for Vec<T> {
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        use std::convert::TryInto;

        let arr = VariantArray::from_variant(variant)?;
        let len: usize = arr
            .len()
            .try_into()
            .expect("variant array length should fit in usize");
        let mut vec = Vec::with_capacity(len);
        for idx in 0..len as i32 {
            let item =
                T::from_variant(&arr.get(idx)).map_err(|e| FromVariantError::InvalidItem {
                    index: idx as usize,
                    error: Box::new(e),
                })?;
            vec.push(item);
        }
        Ok(vec)
    }
}

macro_rules! tuple_length {
    () => { 0usize };
    ($_x:ident, $($xs:ident,)*) => {
        1usize + tuple_length!($($xs,)*)
    };
}

macro_rules! impl_variant_for_tuples_next {
    ($_x:ident, $($xs:ident,)*) => {
        impl_variant_for_tuples!($($xs,)*);
    }
}

macro_rules! impl_variant_for_tuples {
    () => {};
    ( $($name:ident,)+ ) => {
        impl<$($name: ToVariant,)+> ToVariant for ($($name,)+) {
            #[allow(non_snake_case)]
            #[inline]
            fn to_variant(&self) -> Variant {
                let array = VariantArray::new();
                let ($($name,)+) = self;
                $(
                    array.push(&$name.to_variant());
                )+
                array.into_shared().to_variant()
            }
        }

        impl<$($name: FromVariant,)+> FromVariant for ($($name,)+) {
            #[allow(non_snake_case, unused_assignments)]
            #[inline]
            fn from_variant(v: &Variant) -> Result<Self, FromVariantError> {
                let array = VariantArray::from_variant(v)?;
                let expected = tuple_length!($($name,)+);
                let len = array.len() as usize;
                if len != expected {
                    return Err(FromVariantError::InvalidLength { expected, len });
                }

                let mut iter = array.iter();
                let mut index = 0;
                $(
                    let $name = $name::from_variant(&iter.next().unwrap())
                        .map_err(|err| FromVariantError::InvalidItem {
                            index,
                            error: Box::new(err),
                        })?;
                    index += 1;
                )+

                Ok(($($name,)+))
            }
        }

        impl_variant_for_tuples_next!($($name,)+);
    };
}

impl_variant_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,);

godot_test!(
    test_variant_option {
        use std::marker::PhantomData;

        let variant = Some(42_i64).to_variant();
        assert_eq!(Some(42), variant.try_to_i64());

        let variant = Option::<bool>::None.to_variant();
        assert!(variant.is_nil());

        let variant = Variant::new();
        assert_eq!(Ok(None), Option::<i64>::from_variant(&variant));
        assert_eq!(Ok(None), Option::<bool>::from_variant(&variant));
        assert_eq!(Ok(None), Option::<String>::from_variant(&variant));

        let variant = Variant::from_i64(42);
        assert_eq!(Ok(Some(42)), Option::<i64>::from_variant(&variant));
        assert!(Option::<bool>::from_variant(&variant).is_err());
        assert!(Option::<String>::from_variant(&variant).is_err());

        let variant = Variant::new();
        assert_eq!(Ok(Some(())), Option::<()>::from_variant(&variant));
        assert_eq!(Ok(Some(PhantomData)), Option::<PhantomData<*const u8>>::from_variant(&variant));

        let variant = Variant::from_i64(42);
        assert!(Option::<PhantomData<*const u8>>::from_variant(&variant).is_err());
    }

    test_variant_result {
        let variant = Result::<i64, ()>::Ok(42_i64).to_variant();
        let dict = variant.try_to_dictionary().expect("should be dic");
        assert_eq!(Some(42), dict.get("Ok").and_then(|v| v.try_to_i64()));

        let variant = Result::<(), i64>::Err(54_i64).to_variant();
        let dict = variant.try_to_dictionary().expect("should be dic");
        assert_eq!(Some(54), dict.get("Err").and_then(|v| v.try_to_i64()));

        let variant = Variant::from_bool(true);
        assert_eq!(
            Err(FromVariantError::InvalidEnumRepr {
                expected: VariantEnumRepr::ExternallyTagged,
                error: Box::new(FromVariantError::InvalidVariantType {
                    expected: VariantType::Dictionary,
                    variant_type: VariantType::Bool,
                }),
            }),
            Result::<(), i64>::from_variant(&variant),
        );

        let dict = Dictionary::new();
        dict.insert("Ok", 42);
        assert_eq!(Ok(Ok(42)), Result::<i64, i64>::from_variant(&dict.into_shared().to_variant()));

        let dict = Dictionary::new();
        dict.insert("Err", 54);
        assert_eq!(Ok(Err(54)), Result::<i64, i64>::from_variant(&dict.into_shared().to_variant()));
    }

    test_to_variant_iter {
        let slice: &[i64] = &[0, 1, 2, 3, 4];
        let variant = slice.to_variant();
        let array = variant.try_to_array().expect("should be array");
        assert_eq!(5, array.len());
        for i in 0..5 {
            assert_eq!(Some(i), array.get(i as i32).try_to_i64());
        }

        let vec = Vec::<i64>::from_variant(&variant).expect("should succeed");
        assert_eq!(slice, vec.as_slice());

        let het_array = VariantArray::new();
        het_array.push(&Variant::from_i64(42));
        het_array.push(&Variant::new());

        assert_eq!(
            Err(FromVariantError::InvalidItem {
                index: 1,
                error: Box::new(FromVariantError::InvalidVariantType {
                    expected: VariantType::I64,
                    variant_type: VariantType::Nil,
                }),
            }),
            Vec::<i64>::from_variant(&het_array.duplicate().into_shared().to_variant()),
        );

        assert_eq!(Ok(vec![Some(42), None]), Vec::<Option<i64>>::from_variant(&het_array.duplicate().into_shared().to_variant()));

        het_array.push(&f64::to_variant(&54.0));

        assert_eq!(
            Err(FromVariantError::InvalidItem {
                index: 2,
                error: Box::new(FromVariantError::InvalidVariantType {
                    expected: VariantType::I64,
                    variant_type: VariantType::F64,
                }),
            }),
            Vec::<Option<i64>>::from_variant(&het_array.duplicate().into_shared().to_variant()),
        );

        let vec_maybe = Vec::<MaybeNot<i64>>::from_variant(&het_array.into_shared().to_variant()).expect("should succeed");
        assert_eq!(3, vec_maybe.len());
        assert_eq!(Some(&42), vec_maybe[0].as_ref().ok());
        assert_eq!(Some(&Variant::new()), vec_maybe[1].as_ref().err());
        assert_eq!(Some(&f64::to_variant(&54.0)), vec_maybe[2].as_ref().err());
    }

    test_variant_tuple {
        let variant = (42i64, 54i64).to_variant();
        let arr = variant.try_to_array().expect("should be array");
        assert_eq!(Some(42), arr.get(0).try_to_i64());
        assert_eq!(Some(54), arr.get(1).try_to_i64());

        let tuple = <(i64, i64)>::from_variant(&variant);
        assert_eq!(Ok((42, 54)), tuple);
    }

    test_variant_dispatch {
        let variant = 42i64.to_variant();
        if let VariantDispatch::I64(i) = variant.dispatch() {
            assert_eq!(42, i);
        } else {
            panic!("incorrect dispatch type");
        };

        let variant = true.to_variant();
        if let VariantDispatch::Bool(b) = variant.dispatch() {
            assert!(b);
        } else {
            panic!("incorrect dispatch type");
        };

        let variant = 42.to_variant();
        let number_as_float = match variant.dispatch() {
            VariantDispatch::I64(i) => i as f64,
            VariantDispatch::F64(f) => f,
            _ => panic!("not a number"),
        };
        approx::assert_relative_eq!(42.0, number_as_float);
    }
);

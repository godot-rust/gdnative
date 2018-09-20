use super::*;
use std::mem::{transmute, forget};
use std::default::Default;
use std::fmt;

// TODO: implement Debug, PartialEq, etc.

/// A `Variant` can represent many of godot's core types.
///
/// The underlying data can be either stored inline or reference-counted,
/// dependning on the size of the type and whether the it is trivially copyable.
pub struct Variant(pub(crate) sys::godot_variant);

macro_rules! variant_constructors_transmute {
    (
        $(
            $(#[$attr:meta])*
            pub fn $ctor:ident($Type:ty) -> Self as $GdType:ty : $gd_method:ident;
        )*
    ) => (
        $(
            $(#[$attr])*
            pub fn $ctor(val: $Type) -> Variant {
                unsafe {
                    let api = get_api();
                    let mut dest = sys::godot_variant::default();
                    let gd_val: $GdType = transmute(*val);
                    (api.$gd_method)(&mut dest, &gd_val);
                    Variant(dest)
                }
            }
        )*
    )
}

macro_rules! variant_constructors_wrap {
    (
        $(
            $(#[$attr:meta])*
            pub fn $ctor:ident($Type:ty) -> Self as $GdType:ty : $gd_method:ident;
        )*
    ) => (
        $(
            $(#[$attr])*
            pub fn $ctor(val: $Type) -> Variant {
                unsafe {
                    let api = get_api();
                    let mut dest = sys::godot_variant::default();
                    (api.$gd_method)(&mut dest, &val.0);
                    Variant(dest)
                }
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
            pub fn $try_method:ident(&self) -> Option<$TryType:ident> : $try_gd_method:ident;
        )*
    ) => (
        $(
            $(#[$to_attr])*
            pub fn $to_method(&self) -> $ToType {
                unsafe {
                    transmute((get_api().$to_gd_method)(&self.0))
                }
            }

            $(#[$try_attr])*
            pub fn $try_method(&self) -> Option<$TryType> {
                if self.get_type() != VariantType::$TryType {
                    return None;
                }
                unsafe {
                    Some(transmute((get_api().$try_gd_method)(&self.0)))
                }
            }
        )*
    )
}

macro_rules! variant_to_type_wrap {
    (
        $(
            $(#[$to_attr:meta])*
            pub fn $to_method:ident(&self) -> $ToType:ident : $to_gd_method:ident;
            $(#[$try_attr:meta])*
            pub fn $try_method:ident(&self) -> Option<$TryType:ident> : $try_gd_method:ident;
        )*
    ) => (
        $(
            $(#[$to_attr])*
            pub fn $to_method(&self) -> $ToType {
                unsafe {
                    $ToType((get_api().$to_gd_method)(&self.0))
                }
            }

            $(#[$try_attr])*
            pub fn $try_method(&self) -> Option<$TryType> {
                if self.get_type() != VariantType::$TryType {
                    return None;
                }
                unsafe {
                    Some($TryType((get_api().$try_gd_method)(&self.0)))
                }
            }
        )*
    )
}

use sys::godot_variant_type::*;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VariantType {
    Nil = GODOT_VARIANT_TYPE_NIL as u32,
    Bool = GODOT_VARIANT_TYPE_BOOL as u32,
    I64 = GODOT_VARIANT_TYPE_INT as u32,
    F64 = GODOT_VARIANT_TYPE_REAL as u32,
    GodotString = GODOT_VARIANT_TYPE_STRING as u32,
    Vector2 = GODOT_VARIANT_TYPE_VECTOR2 as u32,
    Rect2 = GODOT_VARIANT_TYPE_RECT2 as u32,
    Vector3 = GODOT_VARIANT_TYPE_VECTOR3 as u32,
    Transform2D = GODOT_VARIANT_TYPE_TRANSFORM2D as u32,
    Plane = GODOT_VARIANT_TYPE_PLANE as u32,
    Quat = GODOT_VARIANT_TYPE_QUAT as u32,
    Aabb = GODOT_VARIANT_TYPE_AABB as u32,
    Basis = GODOT_VARIANT_TYPE_BASIS as u32,
    Transform = GODOT_VARIANT_TYPE_TRANSFORM as u32,
    Color = GODOT_VARIANT_TYPE_COLOR as u32,
    NodePath = GODOT_VARIANT_TYPE_NODE_PATH as u32,
    Rid = GODOT_VARIANT_TYPE_RID as u32,
    Object = GODOT_VARIANT_TYPE_OBJECT as u32,
    Dictionary = GODOT_VARIANT_TYPE_DICTIONARY as u32,
    VariantArray = GODOT_VARIANT_TYPE_ARRAY as u32,
    ByteArray = GODOT_VARIANT_TYPE_POOL_BYTE_ARRAY as u32,
    Int32Array = GODOT_VARIANT_TYPE_POOL_INT_ARRAY as u32,
    Float32Array = GODOT_VARIANT_TYPE_POOL_REAL_ARRAY as u32,
    StringArray = GODOT_VARIANT_TYPE_POOL_STRING_ARRAY as u32,
    Vector2Array = GODOT_VARIANT_TYPE_POOL_VECTOR2_ARRAY as u32,
    Vector3Array = GODOT_VARIANT_TYPE_POOL_VECTOR3_ARRAY as u32,
    ColorArray = GODOT_VARIANT_TYPE_POOL_COLOR_ARRAY as u32,
}

impl VariantType {
    #[doc(hidden)]
    pub fn from_sys(v: sys::godot_variant_type) -> VariantType {
        unsafe { transmute(v) }
    }
}

// TODO: Looks like this is missing from the godot_headers bindings.
// It's risky to redefine it here and count on the fact that the integer
// constants will be the same.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VariantOperator {
        //comparison
        Equal, // = OP_EQUAL,
        NotEqual, // = OP_NOT_EQUAL,
        Less, // = OP_LESS,
        LessEqual, // =  OP_LESS_EQUAL,
        Greater, // =  OP_GREATER,
        GreaterEqual, // =  OP_GREATER_EQUAL,
        //mathematic
        Add, // = OP_ADD,
        Subtact, // = OP_SUBTRACT,
        Multiply, // = OP_MULTIPLY,
        Divide, // = OP_DIVIDE,
        Negate, // = OP_NEGATE,
        Positive, // = OP_POSITIVE,
        Module, // = OP_MODULE,
        Concat, // = OP_STRING_CONCAT,
        //bitwise
        ShiftLeft, // = OP_SHIFT_LEFT,
        ShiftRight, // = OP_SHIFT_RIGHT,
        BitAnd, // = OP_BIT_AND,
        BitOr, // = OP_BIT_OR,
        BitXor, // = OP_BIT_XOR,
        BitNegate, // = OP_BIT_NEGATE,
        //logic
        And, // = OP_AND,
        Or, // = OP_OR,
        Xor, // = OP_XOR,
        Not, // = OP_NOT,
        //containment
        In, // = OP_IN,
        Max, // = OP_MAX
}

//fn to_godot_varianty_type(v: VariantType) -> sys::godot_variant_type {
//    unsafe { transmute(v) }
//}

// These aliases are just here so the type name matches the VariantType's variant names
// to make writing macros easier.
type F64 = f64;
type I64 = i64;
type Bool = bool;

impl Variant {

    variant_constructors_transmute!(
        /// Creates a `Variant` wrapping a `Vector2`.
        pub fn from_vector2(&Vector2) -> Self as sys::godot_vector2 : godot_variant_new_vector2;
        /// Creates a `Variant` wrapping a `Vector3`.
        pub fn from_vector3(&Vector3) -> Self as sys::godot_vector3 : godot_variant_new_vector3;
        /// Creates a `Variant` wrapping a `Quat`.
        pub fn from_quat(&Quat) -> Self as sys::godot_quat : godot_variant_new_quat;
        /// Creates a `Variant` wrapping a `Plane`.
        pub fn from_plane(&Plane) -> Self as sys::godot_plane : godot_variant_new_plane;
        /// Creates a `Variant` wrapping a `Rect2`.
        pub fn from_rect2(&Rect2) -> Self as sys::godot_rect2 : godot_variant_new_rect2;
        /// Creates a `Variant` wrapping a `Transform`.
        pub fn from_transform(&Transform) -> Self as sys::godot_transform : godot_variant_new_transform;
        /// Creates a `Variant` wrapping a `Transform2D`.
        pub fn from_transform2d(&Transform2D) -> Self as sys::godot_transform2d : godot_variant_new_transform2d;
        /// Creates a `Variant` wrapping a `Basis`.
        pub fn from_basis(&Basis) -> Self as sys::godot_basis : godot_variant_new_basis;
        /// Creates a `Variant` wrapping a `Color`.
        pub fn from_color(&Color) -> Self as sys::godot_color : godot_variant_new_color;
        /// Creates a `Variant` wrapping an `Aabb`.
        pub fn from_aabb(&Aabb) -> Self as sys::godot_aabb : godot_variant_new_aabb;
    );

    variant_constructors_wrap!(
        /// Creates a `Variant` wrapping an `Rid`.
        pub fn from_rid(&Rid) -> Self as sys::godot_rid : godot_variant_new_rid;
        /// Creates a `Variant` wrapping a `NodePath`.
        pub fn from_node_path(&NodePath) -> Self as sys::godot_node_path : godot_variant_new_node_path;
        /// Creates a `Variant` wrapping a `GodotString`.
        pub fn from_godot_string(&GodotString) -> Self as sys::godot_string : godot_variant_new_string;
        /// Creates an `Variant` wrapping an array of variants.
        pub fn from_array(&VariantArray) -> Self as sys::godot_array : godot_variant_new_array;
        /// Creates a `Variant` wrapping a byte array.
        pub fn from_byte_array(&ByteArray) -> Self as sys::godot_pool_byte_array : godot_variant_new_pool_byte_array;
        /// Creates a `Variant` wrapping an array of 32bit signed integers.
        pub fn from_int32_array(&Int32Array) -> Self as sys::godot_pool_int_array : godot_variant_new_pool_int_array;
        /// Creates a `Variant` wrapping an array of 32bit floats.
        pub fn from_float32_array(&Float32Array) -> Self as sys::godot_pool_real_array : godot_variant_new_pool_real_array;
        /// Creates a `Variant` wrapping an array of godot strings.
        pub fn from_string_array(&StringArray) -> Self as sys::godot_pool_string_array : godot_variant_new_pool_string_array;
        /// Creates a `Variant` wrapping an array of 2d vectors.
        pub fn from_vector2_array(&Vector2Array) -> Self as sys::godot_pool_vector2_array : godot_variant_new_pool_vector2_array;
        /// Creates a `Variant` wrapping an array of 3d vectors.
        pub fn from_vector3_array(&Vector3Array) -> Self as sys::godot_pool_vector3_array : godot_variant_new_pool_vector3_array;
        /// Creates a `Variant` wrapping an array of colors.
        pub fn from_color_array(&ColorArray) -> Self as sys::godot_pool_color_array : godot_variant_new_pool_color_array;
        /// Creates a `Variant` wrapping a dictionary.
        pub fn from_dictionary(&Dictionary) -> Self as sys::godot_dictionary : godot_variant_new_dictionary;
    );

    /// Creates an empty `Variant`.
    pub fn new() -> Self {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_nil)(&mut dest);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a string.
    pub fn from_str<S>(s: S) -> Variant
        where S: AsRef<str>
    {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            let val = s.as_ref();
            let mut godot_s = (api.godot_string_chars_to_utf8_with_len)(val.as_ptr() as *const _, val.len() as _);
            (api.godot_variant_new_string)(&mut dest, &godot_s);
            (api.godot_string_destroy)(&mut godot_s);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a Godot object.
    pub fn from_object<T>(val: &T) -> Variant
        where T: GodotObject
    {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_object)(&mut dest, val.to_sys());
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a signed integer value.
    pub fn from_i64(v: i64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_int)(&mut dest, v);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping an unsigned integer value.
    pub fn from_u64(v: u64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_uint)(&mut dest, v);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping an boolean.
    pub fn from_bool(v: bool) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_bool)(&mut dest, v);
            Variant(dest)
        }
    }

    variant_to_type_transmute!(
        /// Do a best effort to create a `Vector2` out of the variant, possibly returning a default value.
        pub fn to_vector2(&self) -> Vector2 : godot_variant_as_vector2;
        /// Returns `Some(Vector2)` if this variant is one, `None` otherwise.
        pub fn try_to_vector2(&self) -> Option<Vector2> : godot_variant_as_vector2;

        /// Do a best effort to create a `Vector3` out of the variant, possibly returning a default value.
        pub fn to_vector3(&self) -> Vector3 : godot_variant_as_vector3;
        /// Returns `Some(Vector3)` if this variant is one, `None` otherwise.
        pub fn try_to_vector3(&self) -> Option<Vector3> : godot_variant_as_vector3;

        /// Do a best effort to create a `Quat` out of the variant, possibly returning a default value.
        pub fn to_quat(&self) -> Quat : godot_variant_as_quat;
        /// Returns `Some(Quat)` if this variant is one, `None` otherwise.
        pub fn try_to_quat(&self) -> Option<Quat> : godot_variant_as_quat;

        /// Do a best effort to create a `Plane` out of the variant, possibly returning a default value.
        pub fn to_plane(&self) -> Plane : godot_variant_as_plane;
        /// Returns `Some(Plane)` if this variant is one, `None` otherwise.
        pub fn try_to_plane(&self) -> Option<Plane> : godot_variant_as_plane;

        /// Do a best effort to create a `Rect2` out of the variant, possibly returning a default value.
        pub fn to_rect2(&self) -> Rect2 : godot_variant_as_rect2;
        /// Returns `Some(Rect2)` if this variant is one, `None` otherwise.
        pub fn try_to_rect2(&self) -> Option<Rect2> : godot_variant_as_rect2;

        /// Do a best effort to create a `Transform` out of the variant, possibly returning a default value.
        pub fn to_transform(&self) -> Transform : godot_variant_as_transform;
        /// Returns `Some(Transform)` if this variant is one, `None` otherwise.
        pub fn try_to_transform(&self) -> Option<Transform> : godot_variant_as_transform;

        /// Do a best effort to create a `Transform2D` out of the variant, possibly returning a default value.
        pub fn to_transform2d(&self) -> Transform2D : godot_variant_as_transform2d;
        /// Returns `Some(Transform2D)` if this variant is one, `None` otherwise.
        pub fn try_to_transform2d(&self) -> Option<Transform2D> : godot_variant_as_transform2d;

        /// Do a best effort to create a `Basis` out of the variant, possibly returning a default value.
        pub fn to_basis(&self) -> Basis : godot_variant_as_basis;
        /// Returns `Some(Basis)` if this variant is one, `None` otherwise.
        pub fn try_to_basis(&self) -> Option<Basis> : godot_variant_as_basis;

        /// Do a best effort to create a `Color` out of the variant, possibly returning a default value.
        pub fn to_color(&self) -> Color : godot_variant_as_color;
        /// Returns `Some(Color)` if this variant is one, `None` otherwise.
        pub fn try_to_color(&self) -> Option<Color> : godot_variant_as_color;

        /// Do a best effort to create an `Aabb` out of the variant, possibly returning a default value.
        pub fn to_aabb(&self) -> Aabb : godot_variant_as_aabb;
        /// Returns `Some(Aabb)` if this variant is one, `None` otherwise.
        pub fn try_to_aabb(&self) -> Option<Aabb> : godot_variant_as_aabb;

        /// Do a best effort to create a `f64` out of the variant, possibly returning a default value.
        pub fn to_f64(&self) -> F64 : godot_variant_as_real;
        /// Returns `Some(f64)` if this variant is one, `None` otherwise.
        pub fn try_to_f64(&self) -> Option<F64> : godot_variant_as_real;

        /// Do a best effort to create an `i64` out of the variant, possibly returning a default value.
        pub fn to_i64(&self) -> I64 : godot_variant_as_int;
        /// Returns `Some(i64)` if this variant is one, `None` otherwise.
        pub fn try_to_i64(&self) -> Option<I64> : godot_variant_as_int;

        /// Do a best effort to create a `bool` out of the variant, possibly returning a default value.
        pub fn to_bool(&self) -> Bool : godot_variant_as_bool;
        /// Returns `Some(bool)` if this variant is one, `None` otherwise.
        pub fn try_to_bool(&self) -> Option<Bool> : godot_variant_as_bool;
    );

    variant_to_type_wrap!(
        /// Do a best effort to create a `NodePath` out of the variant, possibly returning a default value.
        pub fn to_node_path(&self) -> NodePath : godot_variant_as_node_path;
        /// Returns `Some(NodePath)` if this variant is one, `None` otherwise.
        pub fn try_to_node_path(&self) -> Option<NodePath> : godot_variant_as_node_path;

        /// Do a best effort to create a `GodotString` out of the variant, possibly returning a default value.
        pub fn to_godot_string(&self) -> GodotString : godot_variant_as_string;
        /// Returns `Some(GodotString)` if this variant is one, `None` otherwise.
        pub fn try_to_godot_string(&self) -> Option<GodotString> : godot_variant_as_string;

        /// Do a best effort to create a `Rid` out of the variant, possibly returning a default value.
        pub fn to_rid(&self) -> Rid : godot_variant_as_rid;
        /// Returns `Some(Rid)` if this variant is one, `None` otherwise.
        pub fn try_to_rid(&self) -> Option<Rid> : godot_variant_as_rid;

        /// Do a best effort to create a `VariantArray` out of the variant, possibly returning a default value.
        pub fn to_array(&self) -> VariantArray : godot_variant_as_array;
        /// Returns `Some(VariantArray)` if this variant is one, `None` otherwise.
        pub fn try_to_array(&self) -> Option<VariantArray> : godot_variant_as_array;

        /// Do a best effort to create a `ByteArray` out of the variant, possibly returning a default value.
        pub fn to_byte_array(&self) -> ByteArray : godot_variant_as_pool_byte_array;
        /// Returns `Some(ByteArray)` if this variant is one, `None` otherwise.
        pub fn try_to_byte_array(&self) -> Option<ByteArray> : godot_variant_as_pool_byte_array;

        /// Do a best effort to create an `Int32Array` out of the variant, possibly returning a default value.
        pub fn to_int32_array(&self) -> Int32Array : godot_variant_as_pool_int_array;
        /// Returns `Some(Int32Array)` if this variant is one, `None` otherwise.
        pub fn try_to_int32_array(&self) -> Option<Int32Array> : godot_variant_as_pool_int_array;

        /// Do a best effort to create a `Float32Array` out of the variant, possibly returning a default value.
        pub fn to_float32_array(&self) -> Float32Array : godot_variant_as_pool_real_array;
        /// Returns `Some(Float32Array)` if this variant is one, `None` otherwise.
        pub fn try_to_float32_array(&self) -> Option<Float32Array> : godot_variant_as_pool_real_array;

        /// Do a best effort to create a `StringArray` out of the variant, possibly returning a default value.
        pub fn to_string_array(&self) -> StringArray : godot_variant_as_pool_string_array;
        /// Returns `Some(StringArray)` if this variant is one, `None` otherwise.
        pub fn try_to_string_array(&self) -> Option<StringArray> : godot_variant_as_pool_string_array;

        /// Do a best effort to create a `Vector2Array` out of the variant, possibly returning a default value.
        pub fn to_vector2_array(&self) -> Vector2Array : godot_variant_as_pool_vector2_array;
        /// Returns `Some(Vector2Array)` if this variant is one, `None` otherwise.
        pub fn try_to_vector2_array(&self) -> Option<Vector2Array> : godot_variant_as_pool_vector2_array;

        /// Do a best effort to create a `Vector3Array` out of the variant, possibly returning a default value.
        pub fn to_vector3_array(&self) -> Vector3Array : godot_variant_as_pool_vector3_array;
        /// Returns `Some(Vector3Array)` if this variant is one, `None` otherwise.
        pub fn try_to_vector3_array(&self) -> Option<Vector3Array> : godot_variant_as_pool_vector3_array;

        /// Do a best effort to create a `ColorArray` out of the variant, possibly returning a default value.
        pub fn to_color_array(&self) -> ColorArray : godot_variant_as_pool_color_array;
        /// Returns `Some(ColorArray)` if this variant is one, `None` otherwise.
        pub fn try_to_color_array(&self) -> Option<ColorArray> : godot_variant_as_pool_color_array;

        /// Do a best effort to create a `Dictionary` out of the variant, possibly returning a default value.
        pub fn to_dictionary(&self) -> Dictionary : godot_variant_as_dictionary;
        /// Returns `Some(Dictionary)` if this variant is one, `None` otherwise.
        pub fn try_to_dictionary(&self) -> Option<Dictionary> : godot_variant_as_dictionary;
    );

    pub fn try_to_object<T>(&self) -> Option<T>
        where T: GodotObject
    {
        use sys::godot_variant_type::*;
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(&self.0) != GODOT_VARIANT_TYPE_OBJECT {
                return None;
            }
            let obj = Object::from_sys((api.godot_variant_as_object)(&self.0));
            obj.cast::<T>()
        }
    }

    pub fn to_string(&self) -> String {
        self.to_godot_string().to_string()
    }

    pub fn try_to_string(&self) -> Option<String> {
        self.try_to_godot_string().map(|s|{ s.to_string() })
    }

    /// Returns this variant's type.
    pub fn get_type(&self) -> VariantType {
        unsafe {
            VariantType::from_sys(
                (get_api().godot_variant_get_type)(&self.0)
            )
        }
    }

    /// Returns true if this is an empty variant.
    pub fn is_nil(&self) -> bool {
        self.get_type() == VariantType::Nil
    }

    pub fn has_method(&self, method: &GodotString) -> bool {
        unsafe {
            (get_api().godot_variant_has_method)(&self.0, &method.0)
        }
    }

    // TODO: return a proper error.
    pub fn call(&mut self, method: &GodotString, args: &[Variant]) -> Result<(), ()> {
        unsafe {
            let api = get_api();
            let mut err = sys::godot_variant_call_error::default();
            if args.is_empty() {
                let mut first = ::std::ptr::null() as *const sys::godot_variant;
                (api.godot_variant_call)(
                    &mut self.0,
                    &method.0,
                    &mut first, 0,
                    &mut err
                );
            } else {
                // TODO: double check that this is safe.
                let gd_args: &[sys::godot_variant] = transmute(args);
                let mut first = &gd_args[0] as *const sys::godot_variant;
                (api.godot_variant_call)(
                    &mut self.0,
                    &method.0,
                    &mut first, args.len() as i32,
                    &mut err
                );
            }

            if err.error == sys::godot_variant_call_error_error::GODOT_CALL_ERROR_CALL_OK {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub(crate) fn cast_ref<'l>(ptr: *const sys::godot_variant) -> &'l Variant {
        unsafe { transmute(ptr) }
    }

    pub(crate) fn cast_mut_ref<'l>(ptr: *mut sys::godot_variant) -> &'l mut Variant {
        unsafe { transmute(ptr) }
    }

    /// Returns the internal ffi representation of the variant and consumes
    /// the rust object without running the destructor.
    ///
    /// This should be only used when certain that the receiving side is
    /// responsible for running the destructor for the object, otherwise
    /// it is leaked.
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
    pub fn to_sys(&self) -> sys::godot_variant {
        self.0
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_variant {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_variant) -> Self {
        Variant(sys)
    }
}

impl_basic_traits!(
    for Variant as godot_variant {
        Drop => godot_variant_destroy;
        Clone => godot_variant_new_copy;
        PartialEq => godot_variant_operator_equal;
    }
);

impl Default for Variant {
    fn default() -> Self { Variant::new() }
}

impl fmt::Debug for Variant {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}({})", self.get_type(), self.to_string())
    }
}

macro_rules! variant_from_ref {
    ($(impl From<&$Type:ty> : $ctor:ident;)*) => (
        $(
            impl<'l> From<&'l $Type> for Variant
            {
                fn from(val: &'l $Type) -> Variant {
                    Variant::$ctor(val)
                }
            }
        )*
    );
}

macro_rules! variant_from_val {
    ($(impl From<$Type:ty> : $ctor:ident;)*) => (
        $(
            impl From<$Type> for Variant
            {
                fn from(val: $Type) -> Variant {
                    Variant::$ctor(val)
                }
            }
        )*
    );
}

variant_from_val!(
    impl From<i64> : from_i64;
    impl From<u64> : from_u64;
    impl From<bool> : from_bool;
);

variant_from_ref!(
    impl From<&Vector2> : from_vector2;
    impl From<&Vector3> : from_vector3;
    impl From<&Quat> : from_quat;
    impl From<&Plane> : from_plane;
    impl From<&Rect2> : from_rect2;
    impl From<&Transform> : from_transform;
    impl From<&Transform2D> : from_transform2d;
    impl From<&Basis> : from_basis;
    impl From<&Color> : from_color;
    impl From<&Aabb> : from_aabb;
    impl From<&String> : from_str;
    impl From<&Rid> : from_rid;
    impl From<&NodePath> : from_node_path;
    impl From<&GodotString> : from_godot_string;
    impl From<&Dictionary> : from_dictionary;
    impl From<&VariantArray> : from_array;
    impl From<&ByteArray> : from_byte_array;
    impl From<&Int32Array> : from_int32_array;
    impl From<&Float32Array> : from_float32_array;
    impl From<&Vector2Array> : from_vector2_array;
    impl From<&Vector3Array> : from_vector3_array;
    impl From<&ColorArray> : from_color_array;
);

impl<'l> From<&'l str> for Variant {
    fn from(v: &str) -> Variant {
        Variant::from_str(v)
    }
}

impl <T> From<T> for Variant
    where T: GodotObject
{
    fn from(val: T) -> Variant {
        Variant::from_object(&val)
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

        assert!(!nil.has_method(&GodotString::from_str("foo")));

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
);


/// Types that can be converted to and from a `Variant`.
pub trait ToVariant: Sized {
    fn to_variant(&self) -> Variant;
    fn from_variant(variant: &Variant) -> Option<Self>;
}

impl ToVariant for () {
    fn to_variant(&self) -> Variant {
        Variant::new()
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        if variant.get_type() == VariantType::Nil {
            Some(())
        } else {
            None
        }
    }
}

macro_rules! impl_to_variant_for_int {
    ($ty:ty) => (
        impl ToVariant for $ty {
            fn to_variant(&self) -> Variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_int)(&mut ret, i64::from(*self));
                    Variant(ret)
                }
            }

            fn from_variant(variant: &Variant) -> Option<Self> {
                unsafe {
                    let api = get_api();
                    if (api.godot_variant_get_type)(&variant.0) == sys::godot_variant_type::GODOT_VARIANT_TYPE_INT {
                        Some((api.godot_variant_as_int)(&variant.0) as Self)
                    } else {
                        None
                    }
                }
            }
        }
    )
}

impl_to_variant_for_int!(i8);
impl_to_variant_for_int!(i16);
impl_to_variant_for_int!(i32);
impl_to_variant_for_int!(i64);

macro_rules! godot_uint_impl {
    ($ty:ty) => (
        impl ToVariant for $ty {
            fn to_variant(&self) -> Variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_uint)(&mut ret, u64::from(*self));
                    Variant(ret)
                }
            }

            fn from_variant(variant: &Variant) -> Option<Self> {
                unsafe {
                    let api = get_api();
                    if (api.godot_variant_get_type)(&variant.0) == sys::godot_variant_type::GODOT_VARIANT_TYPE_INT {
                        Some((api.godot_variant_as_uint)(&variant.0) as Self)
                    } else {
                        None
                    }
                }
            }
        }
    )
}

godot_uint_impl!(u8);
godot_uint_impl!(u16);
godot_uint_impl!(u32);
godot_uint_impl!(u64);


impl ToVariant for f32 {
    fn to_variant(&self) -> Variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, f64::from(*self));
            Variant(ret)
        }
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(&variant.0) == sys::godot_variant_type::GODOT_VARIANT_TYPE_REAL {
                Some((api.godot_variant_as_real)(&variant.0) as Self)
            } else {
                None
            }
        }
    }
}

impl ToVariant for f64 {
    fn to_variant(&self) -> Variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, *self);
            Variant(ret)
        }
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(&variant.0) == sys::godot_variant_type::GODOT_VARIANT_TYPE_REAL {
                Some((api.godot_variant_as_real)(&variant.0) as Self)
            } else {
                None
            }
        }
    }
}

impl ToVariant for String {
    fn to_variant(&self) -> Variant {
        Variant::from_str(&self)
    }

    fn from_variant(variant: &Variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(&variant.0) == sys::godot_variant_type::GODOT_VARIANT_TYPE_STRING {
                let mut gd_variant = (api.godot_variant_as_string)(&variant.0);
                let tmp = (api.godot_string_utf8)(&gd_variant);
                let ret = ::std::ffi::CStr::from_ptr((api.godot_char_string_get_data)(&tmp) as *const _)
                    .to_string_lossy()
                    .into_owned();
                (api.godot_string_destroy)(&mut gd_variant);
                Some(ret)
            } else {
                None
            }
        }
    }
}


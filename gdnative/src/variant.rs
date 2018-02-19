use super::*;
use std::mem::{transmute, forget};
use std::default::Default;

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
            $(#[$attr:meta])*
            pub fn $method:ident(&self) -> Option<$Type:ident> : $gd_method:ident;
        )*
    ) => (
        $(
            $(#[$attr])*
            pub fn $method(&self) -> Option<$Type> {
                if self.get_type() != VariantType::$Type {
                    return None;
                }
                unsafe {
                    Some(transmute((get_api().$gd_method)(&self.0)))
                }
            }
        )*
    )
}

macro_rules! variant_to_type_wrap {
    (
        $(
            $(#[$attr:meta])*
            pub fn $method:ident(&self) -> Option<$Type:ident> : $gd_method:ident;
        )*
    ) => (
        $(
            $(#[$attr])*
            pub fn $method(&self) -> Option<$Type> {
                if self.get_type() != VariantType::$Type {
                    return None;
                }
                unsafe {
                    Some($Type((get_api().$gd_method)(&self.0)))
                }
            }
        )*
    )
}

use sys::godot_variant_type::*;

// TODO: reimplementing this enum here will let us add support for things like
// serde and have more idiomatic names, but it might preferrable to generate
// this directly with bindgen (if that's possible).

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
    Array = GODOT_VARIANT_TYPE_ARRAY as u32,
    PoolByteArray = GODOT_VARIANT_TYPE_POOL_BYTE_ARRAY as u32,
    I64Array = GODOT_VARIANT_TYPE_POOL_INT_ARRAY as u32,
    PoolF32Array = GODOT_VARIANT_TYPE_POOL_REAL_ARRAY as u32,
    PoolStringArray = GODOT_VARIANT_TYPE_POOL_STRING_ARRAY as u32,
    PoolVector2Array = GODOT_VARIANT_TYPE_POOL_VECTOR2_ARRAY as u32,
    PoolVector3Array = GODOT_VARIANT_TYPE_POOL_VECTOR3_ARRAY as u32,
    PoolColorArray = GODOT_VARIANT_TYPE_POOL_COLOR_ARRAY as u32,
}

fn from_godot_varianty_type(v: sys::godot_variant_type) -> VariantType {
    unsafe { transmute(v) }
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
        pub fn from_array(&Array) -> Self as sys::godot_array : godot_variant_new_array;
        /// Creates a `Variant` wrapping a byte array.
        pub fn from_pool_byte_array(&PoolByteArray) -> Self as sys::godot_pool_byte_array : godot_variant_new_pool_byte_array;
        /// Creates a `Variant` wrapping an array of godot strings.
        pub fn from_pool_string_array(&PoolStringArray) -> Self as sys::godot_pool_string_array : godot_variant_new_pool_string_array;
        /// Creates a `Variant` wrapping an array of Vector2.
        pub fn from_pool_vector2_array(&PoolVector2Array) -> Self as sys::godot_pool_vector2_array : godot_variant_new_pool_vector2_array;
        /// Creates a `Variant` wrapping an array of Vector3.
        pub fn from_pool_vector3_array(&PoolVector3Array) -> Self as sys::godot_pool_vector3_array : godot_variant_new_pool_vector3_array;
        /// Creates a `Variant` wrapping a dictionary.
        pub fn from_dictionary(&Dictionary) -> Self as sys::godot_dictionary : godot_variant_new_dictionary;
        // TODO: missing C binding?
        // /// Creates a `Variant` wrapping a `StringName`.
        // pub fn from_string_name(&StringName) -> Self as sys::godot_string_name : godot_variant_new_string_name;
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
    pub fn from_object<T>(o: GodotRef<T>) -> Variant
        where T: GodotClass
    {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_object)(&mut dest, o.this);
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
        /// Returns `Some(Vector2)` if this variant is one, `None` otherwise.
        pub fn to_vector2(&self) -> Option<Vector2> : godot_variant_as_vector2;
        /// Returns `Some(Vector3)` if this variant is one, `None` otherwise.
        pub fn to_vector3(&self) -> Option<Vector3> : godot_variant_as_vector3;
        /// Returns `Some(Quat)` if this variant is one, `None` otherwise.
        pub fn to_quat(&self) -> Option<Quat> : godot_variant_as_quat;
        /// Returns `Some(Plane)` if this variant is one, `None` otherwise.
        pub fn to_plane(&self) -> Option<Plane> : godot_variant_as_plane;
        /// Returns `Some(Rect2)` if this variant is one, `None` otherwise.
        pub fn to_rect2(&self) -> Option<Rect2> : godot_variant_as_rect2;
        /// Returns `Some(Transform)` if this variant is one, `None` otherwise.
        pub fn to_transform(&self) -> Option<Transform> : godot_variant_as_transform;
        /// Returns `Some(Transform2D)` if this variant is one, `None` otherwise.
        pub fn to_transform2d(&self) -> Option<Transform2D> : godot_variant_as_transform2d;
        /// Returns `Some(Basis)` if this variant is one, `None` otherwise.
        pub fn to_basis(&self) -> Option<Basis> : godot_variant_as_basis;
        /// Returns `Some(Color)` if this variant is one, `None` otherwise.
        pub fn to_color(&self) -> Option<Color> : godot_variant_as_color;
        /// Returns `Some(Aabb)` if this variant is one, `None` otherwise.
        pub fn to_aabb(&self) -> Option<Aabb> : godot_variant_as_aabb;
        /// Returns `Some(f64)` if this variant is one, `None` otherwise.
        pub fn to_f64(&self) -> Option<F64> : godot_variant_as_real;
        /// Returns `Some(i64)` if this variant is one, `None` otherwise.
        pub fn to_i64(&self) -> Option<I64> : godot_variant_as_int;
        /// Returns `Some(bool)` if this variant is one, `None` otherwise.
        pub fn to_bool(&self) -> Option<Bool> : godot_variant_as_bool;
    );

    variant_to_type_wrap!(
        /// Returns `Some(NodePath)` if this variant is one, `None` otherwise.
        pub fn to_node_path(&self) -> Option<NodePath> : godot_variant_as_node_path;
        /// Returns `Some(GodotString)` if this variant is one, `None` otherwise.
        pub fn to_godot_string(&self) -> Option<GodotString> : godot_variant_as_string;
        /// Returns `Some(Rid)` if this variant is one, `None` otherwise.
        pub fn to_rid(&self) -> Option<Rid> : godot_variant_as_rid;
        /// Returns `Some(Array)` if this variant is one, `None` otherwise.
        pub fn to_array(&self) -> Option<Array> : godot_variant_as_array;
        /// Returns `Some(PoolByteArray)` if this variant is one, `None` otherwise.
        pub fn to_pool_byte_array(&self) -> Option<PoolByteArray> : godot_variant_as_pool_byte_array;
        /// Returns `Some(PoolStringArray)` if this variant is one, `None` otherwise.
        pub fn to_pool_string_array(&self) -> Option<PoolStringArray> : godot_variant_as_pool_string_array;
        /// Returns `Some(PoolVector2Array)` if this variant is one, `None` otherwise.
        pub fn to_pool_vector2_array(&self) -> Option<PoolVector2Array> : godot_variant_as_pool_vector2_array;
        /// Returns `Some(PoolVector3Array)` if this variant is one, `None` otherwise.
        pub fn to_pool_vector3_array(&self) -> Option<PoolVector3Array> : godot_variant_as_pool_vector3_array;
        /// Returns `Some(Dictionary)` if this variant is one, `None` otherwise.
        pub fn to_dictionary(&self) -> Option<Dictionary> : godot_variant_as_dictionary;
        // TODO: missing C binding?
        // /// Returns `Some(StringName)` if this variant is one, `None` otherwise.
        // pub fn to_string_name(&self) -> Option<StringName> : godot_variant_as_string_name;
    );

    pub fn as_object<T>(&self) -> Option<GodotRef<T>>
        where T: GodotClass
    {
        use sys::godot_variant_type::*;
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(&self.0) == GODOT_VARIANT_TYPE_OBJECT {
                let obj = GodotRef::<Object>::from_raw((api.godot_variant_as_object)(&self.0));
                obj.cast::<T>()
            } else {
                None
            }
        }
    }

    /// Returns this variant's type.
    pub fn get_type(&self) -> VariantType {
        unsafe {
            from_godot_varianty_type(
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

    pub fn forget(self) -> sys::godot_variant {
        let v = self.0;
        forget(self);
        v
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
);

impl<'l> From<&'l str> for Variant {
    fn from(v: &str) -> Variant {
        Variant::from_str(v)
    }
}

impl <T> From<GodotRef<T>> for Variant
    where T: GodotClass
{
    fn from(o: GodotRef<T>) -> Variant {
        Variant::from_object(o)
    }
}

godot_test!(
    test_variant_nil {
        let nil = Variant::new();
        assert_eq!(nil.get_type(), VariantType::Nil);
        assert!(nil.is_nil());

        assert!(nil.to_array().is_none());
        assert!(nil.to_rid().is_none());
        assert!(nil.to_i64().is_none());
        assert!(nil.to_bool().is_none());
        assert!(nil.to_aabb().is_none());
        assert!(nil.to_vector2().is_none());
        assert!(nil.to_basis().is_none());

        assert!(!nil.has_method(&GodotString::from_str("foo")));

        let clone = nil.clone();
        assert!(clone == nil);
    }

    test_variant_i64 {
        let v_42 = Variant::from_i64(42);
        assert_eq!(v_42.get_type(), VariantType::I64);

        assert!(!v_42.is_nil());
        assert_eq!(v_42.to_i64(), Some(42));
        assert!(v_42.to_f64().is_none());
        assert!(v_42.to_array().is_none());

        let v_m1 = Variant::from_i64(-1);
        assert_eq!(v_m1.get_type(), VariantType::I64);

        assert!(!v_m1.is_nil());
        assert_eq!(v_m1.to_i64(), Some(-1));
        assert!(v_m1.to_f64().is_none());
        assert!(v_m1.to_array().is_none());
    }
);

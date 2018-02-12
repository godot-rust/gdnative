use super::*;
use std::mem::transmute;
use std::default::Default;

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
            fn $ctor:ident($Type:ty) -> Self as $GdType:ty : $gd_method:ident;
        )*
    ) => (
        $(
            $(#[$attr])*
            pub fn $ctor(val: $Type) -> Variant {
                unsafe {
                    let api = get_api();
                    let mut dest = sys::godot_variant::default();
                    let gd_val: $GdType = transmute(val);
                    (api.$gd_method)(&mut dest, &gd_val);
                    Variant(dest)
                }
            }
        )*
    )
}

macro_rules! variant_to_type {
    (
        $(
            $(#[$attr:meta])*
            fn $method:ident(&self) -> Option<$Type:ident> : $gd_method:ident;
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
    String = GODOT_VARIANT_TYPE_STRING as u32,
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
    ByteArray = GODOT_VARIANT_TYPE_POOL_BYTE_ARRAY as u32,
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

impl Variant {

    variant_constructors!(
        /// Creates a `Variant` wrapping a `Vector2`.
        fn new_vector2(Vector2) -> Self as sys::godot_vector2 : godot_variant_new_vector2;
        /// Creates a `Variant` wrapping a `Vector3`.
        fn new_vector3(Vector3) -> Self as sys::godot_vector3 : godot_variant_new_vector3;
        /// Creates a `Variant` wrapping a `Quat`.
        fn new_quat(Quat) -> Self as sys::godot_quat : godot_variant_new_quat;
        /// Creates a `Variant` wrapping a `Plane`.
        fn new_plane(Plane) -> Self as sys::godot_plane : godot_variant_new_plane;
        /// Creates a `Variant` wrapping a `Rect2`.
        fn new_rect2(Rect2) -> Self as sys::godot_rect2 : godot_variant_new_rect2;
        /// Creates a `Variant` wrapping a `Transform`.
        fn new_transform(Transform) -> Self as sys::godot_transform : godot_variant_new_transform;
        /// Creates a `Variant` wrapping a `Transform2D`.
        fn new_transform2d(Transform2D) -> Self as sys::godot_transform2d : godot_variant_new_transform2d;
        /// Creates a `Variant` wrapping a `Basis`.
        fn new_basis(Basis) -> Self as sys::godot_basis : godot_variant_new_basis;
        /// Creates a `Variant` wrapping a `Color`.
        fn new_color(Color) -> Self as sys::godot_color : godot_variant_new_color;
        /// Creates a `Variant` wrapping an `Aabb`.
        fn new_aabb(Aabb) -> Self as sys::godot_aabb : godot_variant_new_aabb;
    );

    /// Creates an empty `Variant`.
    pub fn new_nil() -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_nil)(&mut dest);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping a string.
    pub fn new_string<S>(s: S) -> Variant
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
    pub fn new_object<T>(o: GodotRef<T>) -> Variant
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
    pub fn new_i64(v: i64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_int)(&mut dest, v);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping an unsigned integer value.
    pub fn new_u64(v: u64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_uint)(&mut dest, v);
            Variant(dest)
        }
    }

    /// Creates a `Variant` wrapping an boolean.
    pub fn new_bool(v: bool) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_bool)(&mut dest, v);
            Variant(dest)
        }
    }


    variant_to_type!(
        /// Returns `Some(Vector2)` if this variant is one, `None` otherwise.
        fn to_vector2(&self) -> Option<Vector2> : godot_variant_as_vector2;
        /// Returns `Some(Vector3)` if this variant is one, `None` otherwise.
        fn to_vector3(&self) -> Option<Vector3> : godot_variant_as_vector3;
        /// Returns `Some(Quat)` if this variant is one, `None` otherwise.
        fn to_quat(&self) -> Option<Quat> : godot_variant_as_quat;
        /// Returns `Some(Plane)` if this variant is one, `None` otherwise.
        fn to_plane(&self) -> Option<Plane> : godot_variant_as_plane;
        /// Returns `Some(Rect2)` if this variant is one, `None` otherwise.
        fn to_rect2(&self) -> Option<Rect2> : godot_variant_as_rect2;
        /// Returns `Some(Transform)` if this variant is one, `None` otherwise.
        fn to_transform(&self) -> Option<Transform> : godot_variant_as_transform;
        /// Returns `Some(Transform2D)` if this variant is one, `None` otherwise.
        fn to_transform2d(&self) -> Option<Transform2D> : godot_variant_as_transform2d;
        /// Returns `Some(Basis)` if this variant is one, `None` otherwise.
        fn to_basis(&self) -> Option<Basis> : godot_variant_as_basis;
        /// Returns `Some(Color)` if this variant is one, `None` otherwise.
        fn to_color(&self) -> Option<Color> : godot_variant_as_color;
        /// Returns `Some(Aabb)` if this variant is one, `None` otherwise.
        fn to_aabb(&self) -> Option<Aabb> : godot_variant_as_aabb;
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
}

impl Default for Variant {
    fn default() -> Self { Variant::new_nil() }
}

macro_rules! variant_from {
    ($(impl From<$Type:ty> : $ctor:ident,)*) => (
        $(
            impl From<$Type> for Variant
            {
                fn from(val: $Type) -> Variant {
                    Variant::$ctor(val)
                }
            }
        )*
    )
}

variant_from!(
    impl From<i64> : new_i64,
    impl From<u64> : new_u64,
    impl From<bool> : new_bool,
    impl From<Vector2> : new_vector2,
    impl From<Vector3> : new_vector3,
    impl From<Quat> : new_quat,
    impl From<Plane> : new_plane,
    impl From<Rect2> : new_rect2,
    impl From<Transform> : new_transform,
    impl From<Transform2D> : new_transform2d,
    impl From<Basis> : new_basis,
    impl From<Color> : new_color,
    impl From<Aabb> : new_aabb,
    impl From<String> : new_string,
);

impl<'l> From<&'l str> for Variant {
    fn from(v: &str) -> Variant {
        Variant::new_string(v)
    }
}

impl <T> From<GodotRef<T>> for Variant
    where T: GodotClass
{
    fn from(o: GodotRef<T>) -> Variant {
        Variant::new_object(o)
    }
}

impl Clone for Variant {
    fn clone(&self) -> Self {
        unsafe {
            let mut dest = sys::godot_variant::default();
            (get_api().godot_variant_new_copy)(&mut dest, &self.0);
            Variant(dest)
        }
    }
}

impl Drop for Variant {
    fn drop(&mut self) {
        unsafe {
            (get_api().godot_variant_destroy)(&mut self.0);
        }
    }
}

use super::*;

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

macro_rules! godot_int_impl {
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

godot_int_impl!(i8);
godot_int_impl!(i16);
godot_int_impl!(i32);
godot_int_impl!(i64);

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

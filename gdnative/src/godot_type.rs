use super::*;

pub trait GodotType: Sized {
    fn to_variant(&self) -> Variant;
    fn from_variant(variant: &Variant) -> Option<Self>;

    fn to_sys_variant(&self) -> sys::godot_variant {
        self.to_variant().forget()
    }
    fn from_sys_variant(variant: &sys::godot_variant) -> Option<Self> {
        Self::from_variant(Variant::cast_ref(variant))
    }
}

impl GodotType for () {
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
        impl GodotType for $ty {
            fn to_variant(&self) -> Variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_int)(&mut ret, *self as i64);
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
        impl GodotType for $ty {
            fn to_variant(&self) -> Variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_uint)(&mut ret, *self as u64);
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


impl GodotType for f32 {
    fn to_variant(&self) -> Variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, *self as f64);
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

impl GodotType for f64 {
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

impl GodotType for String {
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

pub struct Nothing {
    info: GodotClassInfo,
}

unsafe impl GodotClass for Nothing {
    type Reference = Nothing;
    type ClassData = Nothing;

    fn godot_name() -> &'static str {
        ""
    }

    unsafe fn register_class(_desc: *mut libc::c_void) {
        panic!("Can't register");
    }

    fn godot_info(&self) -> &GodotClassInfo {
        &self.info
    }

    unsafe fn from_object(obj: *mut sys::godot_object) -> Self {
        Nothing {
            info: GodotClassInfo {
                this: obj,
            },
        }
    }
    unsafe fn reference(_this: *mut sys::godot_object, data: &Self::ClassData) -> &Self::Reference {
        data
    }
}


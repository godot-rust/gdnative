use super::*;

pub unsafe trait GodotType: Sized {
    fn as_variant(&self) -> sys::godot_variant;
    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self>;
}

unsafe impl GodotType for () {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_nil)(&mut ret);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            if (get_api().godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_NIL {
                Some(())
            } else {
                None
            }
        }
    }
}

macro_rules! godot_int_impl {
    ($ty:ty) => (
        unsafe impl GodotType for $ty {
            fn as_variant(&self) -> sys::godot_variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_int)(&mut ret, *self as i64);
                    ret
                }
            }

            fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
                unsafe {
                    let api = get_api();
                    if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_INT {
                        Some((api.godot_variant_as_int)(variant) as Self)
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
        unsafe impl GodotType for $ty {
            fn as_variant(&self) -> sys::godot_variant {
                unsafe {
                    let mut ret = sys::godot_variant::default();
                    (get_api().godot_variant_new_uint)(&mut ret, *self as u64);
                    ret
                }
            }

            fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
                unsafe {
                    let api = get_api();
                    if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_INT {
                        Some((api.godot_variant_as_uint)(variant) as Self)
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


unsafe impl GodotType for f32 {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, *self as f64);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_REAL {
                Some((api.godot_variant_as_real)(variant) as Self)
            } else {
                None
            }
        }
    }
}

unsafe impl GodotType for f64 {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            (get_api().godot_variant_new_real)(&mut ret, *self);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_REAL {
                Some((api.godot_variant_as_real)(variant) as Self)
            } else {
                None
            }
        }
    }
}

unsafe impl GodotType for String {
    fn as_variant(&self) -> sys::godot_variant {
        unsafe {
            let mut ret = sys::godot_variant::default();
            let api = get_api();
            let mut string = (api.godot_string_chars_to_utf8_with_len)(self.as_ptr() as *const _, self.len() as _);
            (api.godot_variant_new_string)(&mut ret, &string);
            (api.godot_string_destroy)(&mut string);
            ret
        }
    }

    fn from_variant(variant: &mut sys::godot_variant) -> Option<Self> {
        unsafe {
            let api = get_api();
            if (api.godot_variant_get_type)(variant) == sys::godot_variant_type::GODOT_VARIANT_TYPE_STRING {
                let mut variant = (api.godot_variant_as_string)(variant);
                let tmp = (api.godot_string_utf8)(&variant);
                let ret = ::std::ffi::CStr::from_ptr((api.godot_char_string_get_data)(&tmp) as *const _)
                    .to_string_lossy()
                    .into_owned();
                (api.godot_string_destroy)(&mut variant);
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


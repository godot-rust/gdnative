use super::*;

pub struct Variant(pub(crate) sys::godot_variant);

impl Variant {

    pub fn new_nil() -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_nil)(&mut dest);
            Variant(dest)
        }
    }

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

    pub fn new_int(v: i64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_int)(&mut dest, v);
            Variant(dest)
        }
    }

    pub fn new_uint(v: u64) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_uint)(&mut dest, v);
            Variant(dest)
        }
    }

    pub fn new_bool(v: bool) -> Variant {
        unsafe {
            let api = get_api();
            let mut dest = sys::godot_variant::default();
            (api.godot_variant_new_bool)(&mut dest, v);
            Variant(dest)
        }
    }

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
}

impl <T> From<GodotRef<T>> for Variant
    where T: GodotClass
{
    fn from(o: GodotRef<T>) -> Variant {
        Variant::new_object(o)
    }
}

impl From<i64> for Variant {
    fn from(v: i64) -> Variant {
        Variant::new_int(v)
    }
}

impl Drop for Variant {
    fn drop(&mut self) {
        unsafe {
            (get_api().godot_variant_destroy)(&mut self.0);
        }
    }
}
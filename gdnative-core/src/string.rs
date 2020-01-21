use crate::get_api;
use crate::sys;

use std::cmp::Ordering;
use std::ffi::CStr;
use std::fmt;
use std::mem::{forget, transmute};
use std::ops::Range;
use std::slice;
use std::str;

/// Godot's reference-counted string type.
pub struct GodotString(pub(crate) sys::godot_string);

macro_rules! impl_methods {
    // Methods that return a GodotString:
    (
        $(pub fn $method:ident(&self) -> Self : $gd_method:ident;)*
    ) => {
        $(
            pub fn $method(&self) -> Self {
                unsafe {
                    GodotString((get_api().$gd_method)(&self.0))
                }
            }
        )*
    };

    // Methods that return a basic type:
    (
        $(pub fn $method:ident(&self) -> $Type:ty : $gd_method:ident;)*
    ) => {
        $(
            pub fn $method(&self) -> $Type {
                unsafe { (get_api().$gd_method)(&self.0) }
            }
        )*
    };
}

impl GodotString {
    pub fn new() -> Self {
        GodotString::default()
    }

    pub fn from_str<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        unsafe {
            let api = get_api();
            let val = s.as_ref();
            let godot_s =
                (api.godot_string_chars_to_utf8_with_len)(val.as_ptr() as *const _, val.len() as _);

            GodotString(godot_s)
        }
    }

    pub fn len(&self) -> usize {
        unsafe { (get_api().godot_string_length)(&self.0) as usize }
    }

    impl_methods!(
        pub fn is_empty(&self) -> bool : godot_string_empty;
        pub fn is_numeric(&self) -> bool : godot_string_is_numeric;
        pub fn is_valid_float(&self) -> bool : godot_string_is_valid_float;
        pub fn is_valid_html_color(&self) -> bool : godot_string_is_valid_html_color;
        pub fn is_valid_identifier(&self) -> bool : godot_string_is_valid_identifier;
        pub fn is_valid_integer(&self) -> bool : godot_string_is_valid_integer;
        pub fn is_valid_ip_address(&self) -> bool : godot_string_is_valid_ip_address;
        pub fn is_resource_file(&self) -> bool : godot_string_is_resource_file;
        pub fn is_absolute_path(&self) -> bool : godot_string_is_abs_path;
        pub fn is_relative_path(&self) -> bool : godot_string_is_rel_path;
        pub fn to_f32(&self) -> f32 : godot_string_to_float;
        pub fn to_f64(&self) -> f64 : godot_string_to_double;
        pub fn to_i32(&self) -> i32 : godot_string_to_int;
        pub fn u32_hash(&self) -> u32 : godot_string_hash;
        pub fn u64_hash(&self) -> u64 : godot_string_hash64;
        pub fn hex_to_int(&self) -> i32 : godot_string_hex_to_int;
        pub fn hex_to_int_without_prefix(&self) -> i32 : godot_string_hex_to_int_without_prefix;
    );

    impl_methods!(
        pub fn camelcase_to_underscore(&self) -> Self : godot_string_camelcase_to_underscore;
        pub fn camelcase_to_underscore_lowercased(&self) -> Self : godot_string_camelcase_to_underscore_lowercased;
        pub fn capitalize(&self) -> Self : godot_string_capitalize;
        pub fn to_lowercase(&self) -> Self : godot_string_to_lower;
        pub fn to_uppercase(&self) -> Self : godot_string_to_upper;
        pub fn get_file(&self) -> Self : godot_string_get_file;
        pub fn get_base_dir(&self) -> Self : godot_string_get_base_dir;
        pub fn simplify_path(&self) -> Self : godot_string_simplify_path;
        pub fn sha256_text(&self) -> Self : godot_string_sha256_text;
        pub fn md5_text(&self) -> Self : godot_string_md5_text;
        pub fn c_escape(&self) -> Self : godot_string_c_escape;
        pub fn c_escape_multiline(&self) -> Self : godot_string_c_escape_multiline;
        pub fn c_unescape(&self) -> Self : godot_string_c_unescape;
        pub fn http_escape(&self) -> Self : godot_string_http_escape;
        pub fn http_unescape(&self) -> Self: godot_string_http_unescape;
        pub fn json_escape(&self) -> Self : godot_string_json_escape;
        pub fn xml_escape(&self) -> Self : godot_string_xml_escape;
        pub fn xml_escape_with_quotes(&self) -> Self : godot_string_xml_escape_with_quotes;
        pub fn xml_unescape(&self) -> Self: godot_string_xml_unescape;
        pub fn percent_decode(&self) -> Self : godot_string_percent_decode;
        pub fn percent_encode(&self) -> Self : godot_string_percent_encode;
    );

    pub fn is_valid_hex_number(&self, with_prefix: bool) -> bool {
        unsafe { (get_api().godot_string_is_valid_hex_number)(&self.0, with_prefix) }
    }

    pub fn begins_with(&self, s: &GodotString) -> bool {
        unsafe { (get_api().godot_string_begins_with)(&self.0, &s.0) }
    }

    pub fn ends_with(&self, s: &GodotString) -> bool {
        unsafe { (get_api().godot_string_ends_with)(&self.0, &s.0) }
    }

    pub fn begins_with_c_str(&self, s: &CStr) -> bool {
        unsafe { (get_api().godot_string_begins_with_char_array)(&self.0, s.as_ptr()) }
    }

    pub fn sub_string(&self, range: Range<usize>) -> Self {
        unsafe {
            let count = range.end - range.start;
            GodotString((get_api().godot_string_substr)(
                &self.0,
                range.start as i32,
                count as i32,
            ))
        }
    }

    #[doc(hidden)]
    pub fn to_utf8(&self) -> Utf8String {
        unsafe { Utf8String((get_api().godot_string_utf8)(&self.0)) }
    }

    pub fn find(&self, what: &GodotString) -> i32 {
        unsafe { (get_api().godot_string_find)(&self.0, what.0) }
    }

    pub fn find_from(&self, what: &GodotString, from: i32) -> i32 {
        unsafe { (get_api().godot_string_find_from)(&self.0, what.0, from) }
    }

    pub fn find_last(&self, what: &GodotString) -> i32 {
        unsafe { (get_api().godot_string_find_last)(&self.0, what.0) }
    }

    /// Returns the internal ffi representation of the string and consumes
    /// the rust object without running the destructor.
    ///
    /// This should be only used when certain that the receiving side is
    /// responsible for running the destructor for the object, otherwise
    /// it is leaked.
    pub fn forget(self) -> sys::godot_string {
        let v = self.0;
        forget(self);
        v
    }

    /// Returns a copy of the internal ffi representation of the string.
    ///
    /// The string remains owned by the rust wrapper and the receiver of
    /// the ffi representation should not run its destructor.
    pub fn to_sys(&self) -> sys::godot_string {
        self.0
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_string {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_string) -> Self {
        GodotString(sys)
    }
    // TODO: many missing methods.

    impl_common_methods! {
        /// Creates a new reference to this array.
        pub fn new_ref(&self) -> GodotString : godot_string_new_copy;
    }
}

impl_basic_traits!(
    for GodotString as godot_string {
        Drop => godot_string_destroy;
        Eq => godot_string_operator_equal;
        Default => godot_string_new;
    }
);

impl fmt::Debug for GodotString {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_string().fmt(f)
    }
}

impl ToString for GodotString {
    fn to_string(&self) -> String {
        self.to_utf8().to_string()
    }
}

impl std::hash::Hash for GodotString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.u64_hash());
    }
}

// TODO: Is it useful to expose this type?
// Could just make it an internal detail of how to convert to a rust string.
#[doc(hidden)]
pub struct Utf8String(pub(crate) sys::godot_char_string);

impl Utf8String {
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_char_string_length)(&self.0) }
    }

    fn data(&self) -> &u8 {
        unsafe {
            // casting from *const i8 to &u8
            transmute((get_api().godot_char_string_get_data)(&self.0))
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data(), self.len() as usize) }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub fn to_string(&self) -> String {
        String::from(self.as_str())
    }
}

impl_basic_traits!(
    for Utf8String as godot_char_string {
        Drop => godot_char_string_destroy;
    }
);

impl fmt::Debug for Utf8String {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_string().fmt(f)
    }
}

pub struct StringName(pub(crate) sys::godot_string_name);

impl StringName {
    pub fn from_str<S>(s: S)
    where
        S: AsRef<str>,
    {
        let gd_string = GodotString::from_str(s);
        StringName::from_godot_string(&gd_string);
    }

    pub fn from_c_str(s: &CStr) -> Self {
        unsafe {
            let mut result = sys::godot_string_name::default();
            (get_api().godot_string_name_new_data)(&mut result, s.as_ptr());
            StringName(result)
        }
    }

    pub fn from_godot_string(s: &GodotString) -> Self {
        unsafe {
            let mut result = sys::godot_string_name::default();
            (get_api().godot_string_name_new)(&mut result, &s.0);
            StringName(result)
        }
    }

    pub fn get_hash(&self) -> u32 {
        unsafe { (get_api().godot_string_name_get_hash)(&self.0) }
    }

    pub fn get_name(&self) -> GodotString {
        unsafe { GodotString((get_api().godot_string_name_get_name)(&self.0)) }
    }

    pub fn operator_less(&self, s: &StringName) -> bool {
        unsafe { (get_api().godot_string_name_operator_less)(&self.0, &s.0) }
    }
}

impl_basic_traits! {
    for StringName as godot_string_name {
        Drop => godot_string_name_destroy;
        Eq => godot_string_name_operator_equal;
    }
}

impl fmt::Debug for StringName {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.get_name().to_string().fmt(f)
    }
}

impl PartialOrd for StringName {
    fn partial_cmp(&self, other: &StringName) -> Option<Ordering> {
        unsafe {
            let native = (get_api().godot_string_name_operator_less)(&self.0, &other.0);

            if native {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        }
    }
}

impl<S> From<S> for GodotString
where
    S: AsRef<str>,
{
    fn from(s: S) -> GodotString {
        GodotString::from_str(s)
    }
}

godot_test!(test_string {
    use crate::{GodotString, Variant, VariantType};

    let foo: GodotString = "foo".into();
    assert_eq!(foo.len(), 3);

    let foo2 = foo.new_ref();
    assert!(foo == foo2);

    let variant = Variant::from_godot_string(&foo);
    assert!(variant.get_type() == VariantType::GodotString);

    let variant2: Variant = "foo".into();
    assert!(variant == variant2);

    if let Some(foo_variant) = variant.try_to_godot_string() {
        assert!(foo_variant == foo);
    } else {
        panic!("variant should be a GodotString");
    }

    assert_eq!(foo.to_utf8().as_str(), "foo");
});

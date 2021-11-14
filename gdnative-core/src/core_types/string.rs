use crate::object::NewRef;
use crate::private::get_api;
use crate::sys;

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::fmt;
use std::mem::forget;
use std::ops::{Add, AddAssign, Index, Range};
use std::slice;
use std::str;

/// Godot's reference-counted string type.
///
/// This is the Rust binding of GDScript's `String` type. It represents the native string class
/// used within the Godot engine, and as such has different memory layout and characteristics than
/// `std::string::String`.
///
/// `GodotString` is reference-counted like most types in godot-rust. Thus, its `clone()` method
/// does not return a copy of the string, but simply another instance which shares the same backing
/// string. Furthermore, `GodotString` is immutable and does not offer any write APIs. If you need
/// to modify Godot strings, convert them to Rust strings, perform the modifications and convert back.
/// In GDScript, strings have copy-on-write semantics, which guarantees that `GodotString` instances
/// in Rust are independent of their GDScript counterparts. A modification of a string in GDScript
/// (which was previously passed to Rust) will not be reflected in Rust.
///
/// When interfacing with the Godot engine API through godot-rust, you often have the choice between
/// `std::string::String` and `gdnative::core_types::GodotString`. In user methods that are exposed to
/// Godot through the `#[export]` macro, both types can be used as parameters and return types, and any
/// conversions are done transparently.
/// For auto-generated binding APIs in `gdnative::api`, return types are `GodotString`, but parameters
/// are declared `impl Into<GodotString>`, allowing `String` or `&str` to be passed. In addition, the
/// two types can always be explicitly converted using `GodotString::from_str()` and
/// `GodotString::display/to_string()`.
///
/// As a general guideline, use `GodotString` if:
/// * your strings are very large, so you can avoid copying them
/// * you need specific operations only available in Godot (e.g. `sha256_text()`, `c_escape()`, ...)
/// * you primarily pass them between different Godot APIs, without string processing in user code
///
/// Use Rust's `String` if:
/// * you need to modify the string
/// * you would like to decouple part of your code from Godot (e.g. independent game logic, standalone tests)
/// * you want a standard type for interoperability with third-party code (e.g. `regex` crate)
/// * you have a large number of method calls per string instance (which are more expensive due to indirectly calling into Godot)
/// * you need UTF-8 encoding (`GodotString`'s encoding is platform-dependent and unspecified)
pub struct GodotString(pub(crate) sys::godot_string);

macro_rules! impl_methods {
    // Methods that return a GodotString:
    (
        $(pub fn $method:ident(&self) -> Self : $gd_method:ident;)*
    ) => {
        $(
            #[inline]
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
            #[inline]
            pub fn $method(&self) -> $Type {
                unsafe { (get_api().$gd_method)(&self.0) }
            }
        )*
    };
}

impl GodotString {
    #[inline]
    pub fn new() -> Self {
        GodotString::default()
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
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

    #[inline]
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
        pub fn get_basename(&self) -> Self : godot_string_get_basename;
        pub fn get_extension(&self) -> Self : godot_string_get_extension;
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

    #[inline]
    pub fn is_valid_hex_number(&self, with_prefix: bool) -> bool {
        unsafe { (get_api().godot_string_is_valid_hex_number)(&self.0, with_prefix) }
    }

    #[inline]
    pub fn begins_with(&self, s: &GodotString) -> bool {
        unsafe { (get_api().godot_string_begins_with)(&self.0, &s.0) }
    }

    #[inline]
    pub fn ends_with(&self, s: &GodotString) -> bool {
        unsafe { (get_api().godot_string_ends_with)(&self.0, &s.0) }
    }

    #[inline]
    pub fn begins_with_c_str(&self, s: &CStr) -> bool {
        unsafe { (get_api().godot_string_begins_with_char_array)(&self.0, s.as_ptr()) }
    }

    #[inline]
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
    #[inline]
    pub fn to_utf8(&self) -> Utf8String {
        unsafe { Utf8String((get_api().godot_string_utf8)(&self.0)) }
    }

    #[inline]
    pub fn find(&self, what: &GodotString) -> i32 {
        unsafe { (get_api().godot_string_find)(&self.0, what.0) }
    }

    #[inline]
    pub fn find_from(&self, what: &GodotString, from: i32) -> i32 {
        unsafe { (get_api().godot_string_find_from)(&self.0, what.0, from) }
    }

    #[inline]
    pub fn find_last(&self, what: &GodotString) -> i32 {
        unsafe { (get_api().godot_string_find_last)(&self.0, what.0) }
    }

    /// Returns the internal ffi representation of the string and consumes
    /// the rust object without running the destructor.
    ///
    /// This should be only used when certain that the receiving side is
    /// responsible for running the destructor for the object, otherwise
    /// it is leaked.
    #[doc(hidden)]
    #[inline]
    pub fn forget(self) -> sys::godot_string {
        let v = self.0;
        forget(self);
        v
    }

    /// Returns a copy of the internal ffi representation of the string.
    ///
    /// The string remains owned by the rust wrapper and the receiver of
    /// the ffi representation should not run its destructor.
    #[doc(hidden)]
    #[inline]
    pub fn to_sys(&self) -> sys::godot_string {
        self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_string {
        &self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut sys::godot_string {
        &mut self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_string) -> Self {
        GodotString(sys)
    }

    /// Clones `sys` into a `GodotString` without dropping `sys`
    #[doc(hidden)]
    #[inline]
    pub fn clone_from_sys(sys: sys::godot_string) -> Self {
        let sys_string = GodotString(sys);
        let this = sys_string.clone();
        sys_string.forget();
        this
    }

    // TODO: many missing methods.
}

impl Clone for GodotString {
    #[inline]
    fn clone(&self) -> Self {
        self.new_ref()
    }
}

impl_basic_traits_as_sys!(
    for GodotString as godot_string {
        Drop => godot_string_destroy;
        Eq => godot_string_operator_equal;
        Default => godot_string_new;
        NewRef => godot_string_new_copy;
    }
);

impl fmt::Display for GodotString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let utf8 = self.to_utf8();
        f.write_str(utf8.as_str())
    }
}

impl fmt::Debug for GodotString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_string().fmt(f)
    }
}

impl std::hash::Hash for GodotString {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.u64_hash());
    }
}

impl Add<GodotString> for GodotString {
    type Output = GodotString;
    #[inline]
    fn add(self, other: GodotString) -> GodotString {
        &self + &other
    }
}

impl Add<&GodotString> for &GodotString {
    type Output = GodotString;
    #[inline]
    fn add(self, other: &GodotString) -> GodotString {
        GodotString::from_sys(unsafe { (get_api().godot_string_operator_plus)(&self.0, &other.0) })
    }
}

impl<S> Add<S> for &GodotString
where
    S: AsRef<str>,
{
    type Output = GodotString;
    #[inline]
    fn add(self, other: S) -> GodotString {
        self.add(&GodotString::from_str(other))
    }
}

/// `AddAssign` implementations copy the strings' contents since `GodotString` is immutable.
impl AddAssign<&GodotString> for GodotString {
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        *self = &*self + other;
    }
}

/// `AddAssign` implementations copy the strings' contents since `GodotString` is immutable.
impl AddAssign<GodotString> for GodotString {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self += &other;
    }
}

/// `AddAssign` implementations copy the strings' contents since `GodotString` is immutable.
impl<S> AddAssign<S> for GodotString
where
    S: AsRef<str>,
{
    #[inline]
    fn add_assign(&mut self, other: S) {
        self.add_assign(&GodotString::from_str(other))
    }
}

impl PartialOrd for GodotString {
    #[inline]
    fn partial_cmp(&self, other: &GodotString) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GodotString {
    #[inline]
    fn cmp(&self, other: &GodotString) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if unsafe { (get_api().godot_string_operator_less)(&self.0, &other.0) } {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// Type representing a character in Godot's native encoding. Can be converted to and
/// from `char`. Depending on the platform, this might not always be able to represent
/// a full code point.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
pub struct GodotChar(libc::wchar_t);

/// Error indicating that a `GodotChar` cannot be converted to a `char`.
#[derive(Debug)]
pub enum GodotCharError {
    /// The character cannot be represented as a Unicode code point.
    InvalidCodePoint,
    /// The character's encoding cannot be determined on this platform (`wchar_t` is
    /// not 8, 16, or 32-bits wide).
    UnknownEncoding,
    /// The character is part of an incomplete encoding sequence.
    IncompleteSequence,
}

impl TryFrom<GodotChar> for char {
    type Error = GodotCharError;

    #[inline]
    fn try_from(c: GodotChar) -> Result<Self, GodotCharError> {
        match std::mem::size_of::<libc::wchar_t>() {
            1 => std::char::from_u32(c.0 as u32).ok_or(GodotCharError::IncompleteSequence),
            4 => std::char::from_u32(c.0 as u32).ok_or(GodotCharError::InvalidCodePoint),
            2 => {
                let mut iter = std::char::decode_utf16(std::iter::once(c.0 as u16));
                let c = iter
                    .next()
                    .ok_or(GodotCharError::InvalidCodePoint)?
                    .map_err(|_| GodotCharError::IncompleteSequence)?;

                assert!(
                    iter.next().is_none(),
                    "it should be impossible to decode more than one code point from one u16"
                );

                Ok(c)
            }
            _ => Err(GodotCharError::UnknownEncoding),
        }
    }
}

/// Does a best-effort conversion from `GodotChar` to char. If that is not possible,
/// the implementation returns `false`.
impl PartialEq<char> for GodotChar {
    #[inline]
    fn eq(&self, other: &char) -> bool {
        char::try_from(*self).map_or(false, |this| this == *other)
    }
}

/// The index operator provides a low-level view of characters in Godot's native encoding
/// that doesn't always correspond to Unicode code points one-to-one. This operation goes
/// through FFI. For intensive string operations, consider converting to a Rust `String`
/// first to avoid this cost.
impl Index<usize> for GodotString {
    type Output = GodotChar;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            let c: *const libc::wchar_t =
                (get_api().godot_string_operator_index)(self.sys() as *mut _, index as i32);
            &*(c as *const GodotChar)
        }
    }
}

// TODO: Is it useful to expose this type?
// Could just make it an internal detail of how to convert to a rust string.
#[doc(hidden)]
pub struct Utf8String(pub(crate) sys::godot_char_string);

impl Utf8String {
    #[inline]
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_char_string_length)(&self.0) }
    }

    /// Returns `true` if `self` has a length of zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let data = (get_api().godot_char_string_get_data)(&self.0) as _;
            slice::from_raw_parts(data, self.len() as usize)
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_char_string {
        &self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut sys::godot_char_string {
        &mut self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_char_string) -> Self {
        Self(sys)
    }
}

impl ToString for Utf8String {
    #[inline]
    fn to_string(&self) -> String {
        String::from(self.as_str())
    }
}

impl_basic_traits_as_sys!(
    for Utf8String as godot_char_string {
        Drop => godot_char_string_destroy;
    }
);

impl fmt::Debug for Utf8String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_string().fmt(f)
    }
}

pub struct StringName(pub(crate) sys::godot_string_name);

impl StringName {
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        let gd_string = GodotString::from_str(s);
        StringName::from_godot_string(&gd_string)
    }

    #[inline]
    pub fn from_c_str(s: &CStr) -> Self {
        unsafe {
            let mut result = sys::godot_string_name::default();
            (get_api().godot_string_name_new_data)(&mut result, s.as_ptr());
            StringName(result)
        }
    }

    #[inline]
    pub fn from_godot_string(s: &GodotString) -> Self {
        unsafe {
            let mut result = sys::godot_string_name::default();
            (get_api().godot_string_name_new)(&mut result, &s.0);
            StringName(result)
        }
    }

    #[inline]
    pub fn get_hash(&self) -> u32 {
        unsafe { (get_api().godot_string_name_get_hash)(&self.0) }
    }

    #[inline]
    pub fn get_name(&self) -> GodotString {
        unsafe { GodotString((get_api().godot_string_name_get_name)(&self.0)) }
    }

    #[inline]
    pub fn operator_less(&self, s: &StringName) -> bool {
        unsafe { (get_api().godot_string_name_operator_less)(&self.0, &s.0) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_string_name {
        &self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut sys::godot_string_name {
        &mut self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_string_name) -> Self {
        Self(sys)
    }
}

impl_basic_traits_as_sys! {
    for StringName as godot_string_name {
        Drop => godot_string_name_destroy;
        Eq => godot_string_name_operator_equal;
    }
}

impl fmt::Debug for StringName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.get_name().to_string().fmt(f)
    }
}

impl PartialOrd for StringName {
    #[inline]
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
    #[inline]
    fn from(s: S) -> GodotString {
        GodotString::from_str(s)
    }
}

#[cfg(feature = "serde")]
mod serialize {
    use super::*;
    use serde::{
        de::{Error, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };
    use std::fmt::Formatter;

    impl Serialize for GodotString {
        #[inline]
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&*self.to_string())
        }
    }

    #[cfg(feature = "serde")]
    impl<'de> serialize::Deserialize<'de> for GodotString {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            struct GodotStringVisitor;
            impl<'de> Visitor<'de> for GodotStringVisitor {
                type Value = GodotString;

                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    formatter.write_str("a GodotString")
                }

                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    Ok(GodotString::from(s))
                }
            }

            deserializer.deserialize_str(GodotStringVisitor)
        }
    }
}

godot_test!(test_string {
    use crate::core_types::{GodotString, Variant, VariantType, ToVariant};

    let foo: GodotString = "foo".into();
    assert_eq!(foo.len(), 3);
    assert_eq!(foo.to_string(), String::from("foo"));

    let foo2 = foo.new_ref();
    assert!(foo == foo2);

    let bar: GodotString = "bar".into();
    let qux: GodotString = "qux".into();
    assert_eq!(&bar + &qux, "barqux".into());

    let baz: GodotString = "baz".into();
    assert_eq!(&baz + "corge", "bazcorge".into());

    let mut bar2 = bar.new_ref();
    bar2 += &qux;
    assert_eq!(bar2, "barqux".into());

    let cmp1: GodotString = "foo".into();
    let cmp2: GodotString = "foo".into();
    let cmp3: GodotString = "bar".into();
    assert_eq!(cmp1.cmp(&cmp2), Ordering::Equal, "equal should not be less than");
    assert_eq!(cmp1.cmp(&cmp3), Ordering::Greater, "foo should greater than bar");
    assert_eq!(cmp3.cmp(&cmp1), Ordering::Less, "bar should be less than foo");

    let index_string: GodotString = "bar".into();
    assert_eq!(index_string[0], 'b');
    assert_eq!(index_string[1], 'a');
    assert_eq!(index_string[2], 'r');

    let variant = Variant::new(&foo);
    assert!(variant.get_type() == VariantType::GodotString);

    let variant2: Variant = "foo".to_variant();
    assert!(variant == variant2);

    if let Ok(foo_variant) = variant.try_to::<GodotString>() {
        assert!(foo_variant == foo);
    } else {
        panic!("variant should be a GodotString");
    }

    assert_eq!(foo.to_utf8().as_str(), "foo");
});

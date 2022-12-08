//! Interface to Godot's built-in profiler.

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::time::{Duration, Instant};

use crate::private::try_get_api;

/// A string encoding information about the code being profiled for Godot's built-in profiler.
///
/// The string should be in the form of `{file}::{line_number}::{tag}`, where `tag` is an
/// identifier of the code, usually be the name of the method. None of the substrings should
/// contain `::`.
///
/// To create a `Signature` in the correct form, see [`Signature::new()`] or [`profile_sig!`]. To
/// create a `Signature` from an existing `CStr` or `CString`, see [`Signature::from_raw()`] and
/// [`Signature::from_raw_owned()`].
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Signature<'a> {
    sig: Cow<'a, CStr>,
}

impl<'a> Signature<'a> {
    /// Creates a `Signature` from a `CStr` in the specified format. The format is not
    /// checked.
    ///
    /// Adding profiling data using an invalid `Signature` may cause incorrect information to
    /// show up in the editor.
    #[inline(always)]
    pub const fn from_raw(sig: &'a CStr) -> Self {
        Signature {
            sig: Cow::Borrowed(sig),
        }
    }

    /// Creates a `Signature` from a NUL-terminated byte slice, containing a string in the
    /// specified format. Neither the format nor whether the slice is correctly NUL-terminated
    /// is checked.
    ///
    /// This is a convenience method for `Signature::from_raw(CStr::from_bytes_with_nul_unchecked(bytes))`.
    ///
    /// Adding profiling data using an invalid `Signature` may cause incorrect information to
    /// show up in the editor.
    ///
    /// # Safety
    ///
    /// This function will cast the provided `bytes` to a `CStr` wrapper without performing any
    /// sanity checks. The provided slice **must** be nul-terminated and not contain any
    /// interior nul bytes.
    #[inline(always)]
    pub unsafe fn from_bytes_with_nul_unchecked(bytes: &'a [u8]) -> Self {
        let sig = CStr::from_bytes_with_nul_unchecked(bytes);
        Self::from_raw(sig)
    }

    /// Create a borrowed version of `self` for repeated use with [`add_data()`][Self::add_data()] or [`profile()`][Self::add_data()].
    #[inline(always)]
    pub fn borrow(&self) -> Signature<'_> {
        Signature {
            sig: Cow::Borrowed(&*self.sig),
        }
    }

    /// Add a data point to Godot's built-in profiler using this signature.
    ///
    /// See the free function [`profiler::add_data()`][add_data()].
    #[inline]
    pub fn add_data(&self, time: Duration) {
        add_data(self.borrow(), time)
    }

    /// Times a closure and adds the measured time to Godot's built-in profiler with this
    /// signature, and then returns it's return value.
    ///
    /// See the free function [`profiler::profile()`][profile()].
    #[inline]
    pub fn profile<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        profile(self.borrow(), f)
    }

    fn as_ptr(&self) -> *const libc::c_char {
        self.sig.as_ptr()
    }
}

impl Signature<'static> {
    /// Creates a `Signature` in the correct form using the given variables. The format is
    /// checked at runtime.
    ///
    /// # Panics
    ///
    /// If `file` or `tag` contain `::` or NUL-bytes.
    #[inline]
    pub fn new(file: &str, line: u32, tag: &str) -> Self {
        if file.contains("::") {
            panic!("file name should not contain `::`");
        }

        if tag.contains("::") {
            panic!("tag should not contain `::`");
        }

        let sig = CString::new(format!("{file}::{line}::{tag}"))
            .expect("file and tag should not contain NUL bytes");
        Self::from_raw_owned(sig)
    }

    /// Creates a `Signature` from an owned `CString` in the specified format. The format is not
    /// checked.
    ///
    /// Adding profiling data using an invalid `Signature` may cause incorrect information to
    /// show up in the editor.
    #[inline(always)]
    pub const fn from_raw_owned(sig: CString) -> Self {
        Signature {
            sig: Cow::Owned(sig),
        }
    }
}

/// Add a data point to Godot's built-in profiler. The profiler only has microsecond precision.
/// Sub-microsecond time is truncated.
///
/// If the GDNative API is not initialized at the point when this is called, the function will
/// fail silently.
///
/// # Panics
///
/// If the number of microseconds in `time` exceeds the range of `u64`.
#[inline]
pub fn add_data(signature: Signature<'_>, time: Duration) {
    if let Some(api) = try_get_api() {
        let time_in_usec = u64::try_from(time.as_micros())
            .expect("microseconds in `time` should not exceed the range of u64");

        unsafe {
            (api.godot_nativescript_profiling_add_data)(signature.as_ptr(), time_in_usec);
        }
    }
}

/// Times a closure and adds the measured time to Godot's built-in profiler with the given
/// signature, and then returns it's return value.
#[inline]
pub fn profile<F, R>(signature: Signature<'_>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let ret = f();
    add_data(signature, Instant::now() - start);
    ret
}

/// Convenience macro to create a profiling signature with a given tag.
///
/// The expanded code will panic at runtime if the file name or `tag` contains `::` or
/// any NUL-bytes.
///
/// See [`Signature`] for more information.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// use gdnative::profiler::{profile, profile_sig};
///
/// let answer = profile(profile_sig!("foo"), || 42);
/// assert_eq!(answer, 42);
/// # }
/// ```
#[macro_export]
macro_rules! _profile_sig {
    ($tag:expr) => {
        $crate::profiler::Signature::new(file!(), line!(), $tag)
    };
}

// Export macro in this module
pub use _profile_sig as profile_sig;

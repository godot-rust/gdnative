use std::ffi::{CStr, CString};
use std::time::Instant;

use crate::private::get_api;

/// Low level function to add data to Godot's built-in profiler.
///
/// For a high level interface see the [`profile!`] macro.
///
/// The exact formatting of `signature` is undocumented, but it seems to accept the pattern:
/// `"<FILE>::<LINE-NUMBER>::<TAG>"`, where `<TAG>` is the identifier appearing in the profiler.
/// In GDScript the tags corresponds directly to function names, but for native code, one is
/// free to use any naming convention.
#[inline]
pub fn add_data(signature: &CStr, time_in_usec: u64) {
    unsafe {
        let api = get_api();
        (api.godot_nativescript_profiling_add_data)(signature.as_ptr(), time_in_usec);
    }
}

#[cfg(not(feature = "no_profiling"))]
pub struct Profiler {
    name: &'static str,
    start_time: Instant,
}

#[cfg(feature = "no_profiling")]
pub struct Profiler;

impl Profiler {
    #[cfg(not(feature = "no_profiling"))]
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Profiler {
            name,
            start_time: Instant::now(),
        }
    }

    #[cfg(feature = "no_profiling")]
    #[inline]
    pub fn new(_name: &'static str) -> Self {
        Profiler
    }
}

impl Drop for Profiler {
    #[cfg(not(feature = "no_profiling"))]
    #[inline]
    fn drop(&mut self) {
        let time_in_usec = self.start_time.elapsed().as_micros() as u64;
        let signature = CString::new(self.name).expect("Failed to create signature CString");
        add_data(&signature, time_in_usec);
    }

    #[cfg(feature = "no_profiling")]
    #[inline]
    fn drop(&mut self) {}
}

/// Helper macro to profile a certain scope. Usage:
///
/// ```
/// profile!(_p, "name_to_appear_in_profiler");
/// ```
///
/// Note measuring mechanism relies on `Drop`, i.e., it measures the time from
/// the `profile!` line up to the point where `drop` is called, which is usually
/// when `_p` goes out of scope.
#[macro_export]
macro_rules! profile {
    ($p:ident, $f:literal) => {
        let s = concat!(file!(), "::", line!(), "::", $f);
        let $p = $crate::nativescript::profiling::Profiler::new(s);
    };
}

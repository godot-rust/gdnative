use std::ffi::{CStr, CString};

use crate::private::get_api;

pub struct ProfileSignature {
    pub file: String,
    pub line: u32,
    pub tag: String,
}

/// Function to add data to Godot's built-in profiler.
#[inline]
pub fn add_data(signature: &ProfileSignature, time_in_usec: u64) {
    let signature = CString::new(format!(
        "{}::{}::{}",
        signature.file, signature.line, signature.tag
    ))
    .expect("Failed to create signature CString");
    add_data_native(&signature, time_in_usec);
}

/// Low level function to add data to Godot's built-in profiler.
///
/// For a high level interface see the [`add_data`].
#[inline]
pub fn add_data_native(signature: &CStr, time_in_usec: u64) {
    unsafe {
        let api = get_api();
        (api.godot_nativescript_profiling_add_data)(signature.as_ptr(), time_in_usec);
    }
}

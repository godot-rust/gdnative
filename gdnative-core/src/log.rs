//! Functions for using the engine's logging system in the editor.
use std::ffi::CStr;
use std::fmt::{self, Display};

// Collection of macros accessing the Godot engine log/print functionality
pub use crate::{godot_dbg, godot_error, godot_print, godot_site, godot_warn};

use crate::core_types::GodotString;
use crate::private;

/// Value representing a call site for errors and warnings. Can be constructed
/// using the [`godot_site`] macro, or manually.
#[derive(Copy, Clone, Debug)]
pub struct Site<'a> {
    file: &'a CStr,
    func: &'a CStr,
    line: u32,
}

impl<'a> Site<'a> {
    /// Construct a new `Site` value using values provided manually.
    #[inline]
    pub const fn new(file: &'a CStr, func: &'a CStr, line: u32) -> Self {
        Site { file, func, line }
    }
}

impl<'a> Default for Site<'a> {
    #[inline]
    fn default() -> Self {
        let unset = unsafe { CStr::from_bytes_with_nul_unchecked(b"<unset>\0") };
        Site::new(unset, unset, 0)
    }
}

impl<'a> Display for Site<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "file {}, {}, line {}",
            self.file.to_string_lossy(),
            self.func.to_string_lossy(),
            self.line
        )
    }
}

/// Print a message to the Godot console.
///
/// Typically, you would use this through the [`godot_print`] macro.
///
/// # Panics
///
/// If the API isn't initialized.
#[inline]
pub fn print<S: Display>(msg: S) {
    unsafe {
        let msg = GodotString::from_str(msg.to_string());
        (private::get_api().godot_print)(&msg.to_sys() as *const _);
    }
}

/// Print a warning to the Godot console.
///
/// Typically, you would use this through the [`godot_warn`] macro.
///
/// # Panics
///
/// If the API isn't initialized, or if the message contains any NUL-bytes.
#[inline]
pub fn warn<S: Display>(site: Site<'_>, msg: S) {
    let msg = msg.to_string();
    let msg = ::std::ffi::CString::new(msg).unwrap();

    unsafe {
        (private::get_api().godot_print_warning)(
            msg.as_ptr(),
            site.func.as_ptr(),
            site.file.as_ptr(),
            site.line as libc::c_int,
        );
    }
}

/// Print an error to the Godot console.
///
/// Typically, you would use this through the [`godot_error`] macro.
///
/// # Panics
///
/// If the API isn't initialized, or if the message contains any NUL-bytes.
#[inline]
pub fn error<S: Display>(site: Site<'_>, msg: S) {
    let msg = msg.to_string();
    let msg = ::std::ffi::CString::new(msg).unwrap();

    unsafe {
        (private::get_api().godot_print_error)(
            msg.as_ptr(),
            site.func.as_ptr(),
            site.file.as_ptr(),
            site.line as libc::c_int,
        );
    }
}

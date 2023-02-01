/// A handle passed from the engine during NativeScript termination. It's purpose is currently
/// unclear.
pub struct TerminateHandle {
    _handle: *mut libc::c_void,
}

impl TerminateHandle {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(handle: *mut libc::c_void) -> Self {
        TerminateHandle { _handle: handle }
    }
}

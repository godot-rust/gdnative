use super::{GDNativeCallbacks, TerminateHandle};

pub trait TheGDNativeCallbacksAttributeIsRequired {}

#[inline]
pub unsafe fn gdnative_init<C: GDNativeCallbacks>(
    options: *mut crate::sys::godot_gdnative_init_options,
) {
    if !crate::private::bind_api(options) {
        // Can't use godot_error here because the API is not bound.
        // Init errors should be reported by bind_api.
        return;
    }

    crate::init::diagnostics::godot_version_mismatch();

    crate::private::report_panics("gdnative_init", || {
        let init_info = crate::init::InitializeInfo::new(options);
        C::gdnative_init(init_info)
    });
}

#[inline]
pub unsafe fn gdnative_terminate<C: GDNativeCallbacks>(
    options: *mut crate::sys::godot_gdnative_terminate_options,
) {
    if !crate::private::is_api_bound() {
        return;
    }

    crate::private::report_panics("gdnative_terminate", || {
        let term_info = crate::init::TerminateInfo::new(options);
        C::gdnative_terminate(term_info)
    });

    crate::private::cleanup_internal_state();
}

#[inline]
pub unsafe fn gdnative_singleton<C: GDNativeCallbacks>() {
    C::gdnative_singleton();
}

#[inline]
pub unsafe fn nativescript_init<C: GDNativeCallbacks>(handle: *mut libc::c_void) {
    if !crate::private::is_api_bound() {
        return;
    }

    crate::private::report_panics("nativescript_init", || {
        crate::init::auto_register(crate::init::InitHandle::new(
            handle,
            crate::init::InitLevel::AUTO,
        ));
        C::nativescript_init(crate::init::InitHandle::new(
            handle,
            crate::init::InitLevel::USER,
        ));

        crate::init::diagnostics::missing_suggested_diagnostics();
    });
}

#[inline]
pub unsafe fn nativescript_terminate<C: GDNativeCallbacks>(handle: *mut libc::c_void) {
    C::nativescript_terminate(TerminateHandle::new(handle));
}

#[inline]
pub unsafe fn nativescript_frame<C: GDNativeCallbacks>() {
    C::nativescript_frame();
}

#[inline]
pub unsafe fn nativescript_thread_enter<C: GDNativeCallbacks>() {
    C::nativescript_thread_enter();
}

#[inline]
pub unsafe fn nativescript_thread_exit<C: GDNativeCallbacks>() {
    C::nativescript_thread_exit();
}

use crate::core_types::GodotString;

pub struct TerminateInfo {
    in_editor: bool,
}

impl TerminateInfo {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(options: *mut crate::sys::godot_gdnative_terminate_options) -> Self {
        assert!(!options.is_null(), "options were NULL");

        let crate::sys::godot_gdnative_terminate_options { in_editor } = *options;

        Self { in_editor }
    }

    /// Returns `true` if the library is loaded in the Godot Editor.
    #[inline]
    pub fn in_editor(&self) -> bool {
        self.in_editor
    }
}

pub struct InitializeInfo {
    in_editor: bool,
    active_library_path: GodotString,
    options: *mut crate::sys::godot_gdnative_init_options,
}

impl InitializeInfo {
    /// Returns true if the library is loaded in the Godot Editor.
    #[inline]
    pub fn in_editor(&self) -> bool {
        self.in_editor
    }

    /// Returns a path to the library relative to the project.
    ///
    /// Example: `res://../../target/debug/libhello_world.dylib`
    #[inline]
    pub fn active_library_path(&self) -> &GodotString {
        &self.active_library_path
    }

    /// # Safety
    ///
    /// Will `panic!()` if options is NULL or invalid.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(options: *mut crate::sys::godot_gdnative_init_options) -> Self {
        assert!(!options.is_null(), "options were NULL");
        let crate::sys::godot_gdnative_init_options {
            in_editor,
            active_library_path,
            ..
        } = *options;

        let active_library_path =
            crate::core_types::GodotString::clone_from_sys(*active_library_path);

        Self {
            options,
            in_editor,
            active_library_path,
        }
    }

    #[inline]
    pub fn report_loading_error<T>(&self, message: T)
    where
        T: std::fmt::Display,
    {
        let crate::sys::godot_gdnative_init_options {
            report_loading_error,
            gd_native_library,
            ..
        } = unsafe { *self.options };

        if let Some(report_loading_error_fn) = report_loading_error {
            // Add the trailing zero and convert Display => String
            let message = format!("{}\0", message);

            // Convert to FFI compatible string
            let message = std::ffi::CStr::from_bytes_with_nul(message.as_bytes())
                .expect("message should not have a NULL");

            unsafe {
                report_loading_error_fn(gd_native_library, message.as_ptr());
            }
        }
    }
}

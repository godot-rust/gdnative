#![macro_use]

#[macro_export]
macro_rules! godot_init {
    (
        $(
            $class:ty
        ),*
    ) => (
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn godot_gdnative_init(options: *mut $crate::sys::godot_gdnative_init_options) {
            unsafe {
                $crate::GODOT_API = Some($crate::GodotApi::from_raw((*options).api_struct));
            }
        }

        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn godot_gdnative_terminate(_options: *mut $crate::sys::godot_gdnative_terminate_options) {
            unsafe {
                $crate::GODOT_API = None;
            }
        }

        #[no_mangle]
        #[doc(hidden)]
        #[allow(unused_unsafe, unused_variables)]
        pub extern "C" fn godot_nativescript_init(desc: *mut $crate::libc::c_void) {
            unsafe {
                $(
                    <$class as $crate::GodotClass>::register_class(desc);
                )*
            }
        }
    )
}

#[macro_export]
macro_rules! godot_warn {
    ($($args:tt)*) => ({
        let msg = format!($($args)*);
        let line = line!();
        let file = file!();
        #[allow(unused_unsafe)]
        unsafe {
            let msg = ::std::ffi::CString::new(msg).unwrap();
            let file = ::std::ffi::CString::new(file).unwrap();
            let func = b"<native>\0";
            ($crate::get_api().godot_print_warning)(
                msg.as_ptr() as *const _,
                func.as_ptr() as *const _,
                file.as_ptr() as *const _,
                line as _,
            );
        }
    })
}

#[macro_export]
macro_rules! godot_error {
    ($($args:tt)*) => ({
        let msg = format!($($args)*);
        let line = line!();
        let file = file!();
        #[allow(unused_unsafe)]
        unsafe {
            let msg = ::std::ffi::CString::new(msg).unwrap();
            let file = ::std::ffi::CString::new(file).unwrap();
            let func = b"<native>\0";
            ($crate::get_api().godot_print_error)(
                msg.as_ptr() as *const _,
                func.as_ptr() as *const _,
                file.as_ptr() as *const _,
                line as _,
            );
        }
    })
}

macro_rules! impl_basic_trait {
    (
        Drop for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl Drop for $Type {
            fn drop(&mut self) {
                unsafe {
                    (get_api().$gd_method)(&mut self.0)
                }
            }
        }
    };

    (
        Clone for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl Clone for $Type {
            fn clone(&self) -> Self {
               unsafe {
                    let mut result = sys::$GdType::default();
                    (get_api().$gd_method)(&mut result, &self.0);
                    $Type(result)
                }
            }
        }
    };

    (
        Default for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl Default for $Type {
            fn default() -> Self {
                unsafe {
                    let mut gd_val = sys::$GdType::default();
                    (get_api().$gd_method)(&mut gd_val);
                    $Type(gd_val)
                }
            }
        }
    };

    (
        PartialEq for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            fn eq(&self, other: &Self) -> bool {
                unsafe {
                    (get_api().$gd_method)(&self.0, &other.0)
                }
            }
        }
    };

    (
        Eq for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            fn eq(&self, other: &Self) -> bool {
                unsafe {
                    (get_api().$gd_method)(&self.0, &other.0)
                }
            }
        }
        impl Eq for $Type {}
    };
}

macro_rules! impl_basic_traits {
    (
        for $Type:ident as $GdType:ident {
            $( $Trait:ident => $gd_method:ident; )*
        }
    ) => (
        $(
            impl_basic_trait!(
                $Trait for $Type as $GdType : $gd_method
            );
        )*
    )
}

macro_rules! impl_common_method {
    (
        $(#[$attr:meta])*
        pub fn new_ref(&self) -> $Type:ident : $gd_method:ident
    ) => {
        $(#[$attr])*
        pub fn new_ref(&self) -> $Type {
            unsafe {
                let mut result = Default::default();
                (get_api().$gd_method)(&mut result, &self.0);
                $Type(result)
            }
        }
    };
}

macro_rules! impl_common_methods {
    (
        $(
            $(#[$attr:meta])*
            pub fn $name:ident(&self $(,$pname:ident : $pty:ty)*) -> $Ty:ident : $gd_method:ident;
        )*
    ) => (
        $(
            $(#[$attr])*
            impl_common_method!(
                pub fn $name(&self $(,$pname : $pty)*) -> $Ty : $gd_method
            );
        )*
    )
}


macro_rules! godot_test {
    ($($test_name:ident $body:block)*) => {
        $(
            #[cfg(feature = "gd_test")]
            pub fn $test_name() -> bool {
                let str_name = stringify!($test_name);
                println!("   -- {}", str_name);

                let ok = ::std::panic::catch_unwind(
                    || $body
                ).is_ok();

                if !ok {
                    godot_error!("   !! Test {} failed", str_name);
                }

                ok
            }
        )*
    }
}

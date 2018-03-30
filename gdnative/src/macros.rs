#![macro_use]


#[macro_export]
macro_rules! godot_gdnative_init {
    ($name:ident) => {
        godot_gdnative_init!($name, |_| {});
    };
    ($name:ident, $fun:expr) => {
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn $name(options: *mut $crate::sys::godot_gdnative_init_options) {
            unsafe {
                $crate::GODOT_API = Some($crate::GodotApi::from_raw((*options).api_struct));
            }
            let api = $crate::get_api();
            // Force the initialization of the method table of common types. This way we can
            // assume that if the api object is alive we can fetch the method of these types
            // without checking for initialization.
            $crate::ReferenceMethodTable::get(api);

            $fun(options);
        }
    };
}

#[macro_export]
macro_rules! godot_gdnative_terminate {
    ($name:ident) => {
        godot_gdnative_terminate!($name, |_| {});
    };
    ($name:ident, $fun:expr) => {
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn $name(options: *mut $crate::sys::godot_gdnative_terminate_options) {
            $fun(options);

            unsafe {
                $crate::GODOT_API = None;
            }
        }
    };
}

#[macro_export]
macro_rules! godot_nativescript_init {
    ($name:ident) => {
        godot_nativescript_init!($name, |_| {});
    };
    ($name:ident, $fun:expr) => {
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn $name(handle: *mut $crate::libc::c_void) {
            $fun(handle);
        }
    };
}

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
                let api = $crate::get_api();
                // Force the initialization of the method table of common types. This way we can
                // assume that if the api object is alive we can fetch the method of these types
                // without checking for initialization.
                $crate::ReferenceMethodTable::get(api);
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
                    <$class as $crate::NativeClass>::register_class(desc);
                )*
            }
        }
    )
}

#[macro_export]
macro_rules! godot_print {
    ($($args:tt)*) => ({
        let msg = format!($($args)*);

        #[allow(unused_unsafe)]
        unsafe {
            let msg = $crate::GodotString::from_str(msg);
            ($crate::get_api().godot_print)(&msg.to_sys() as *const _);
        }
    });
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


#[macro_export]
macro_rules! godot_wrap_constructor {
    ($_name:ty, $c:expr) => {
        {
            unsafe extern "C" fn constructor(
                this: *mut $crate::sys::godot_object,
                _method_data: *mut $crate::libc::c_void
            ) -> *mut $crate::libc::c_void {
                use std::cell::RefCell;
                use std::boxed::Box;

                let val = $c($crate::NativeInstanceHeader{ this: this });

                let wrapper = Box::new(RefCell::new(val));
                Box::into_raw(wrapper) as *mut _
            }

            constructor
        }
    }
}

#[macro_export]
macro_rules! godot_wrap_destructor {
    ($name:ty) => {
        {
            #[allow(unused_unsafe)]
            unsafe extern "C" fn destructor(
                _this: *mut $crate::sys::godot_object,
                _method_data: *mut $crate::libc::c_void,
                user_data: *mut $crate::libc::c_void
            ) -> () {
                use std::cell::RefCell;
                use std::boxed::Box;

                let wrapper: Box<RefCell<$name>> = unsafe { Box::from_raw(user_data as *mut _) };
                drop(wrapper)
            }

            destructor
        }
    }
}

#[macro_export]
macro_rules! godot_wrap_method_parameter_count {
    () => {
        0
    };
    ($name:ident, $($other:ident,)*) => {
        1 + godot_wrap_method_parameter_count!($($other,)*)
    }
}

#[macro_export]
macro_rules! godot_wrap_method {
    (
        $type_name:ty,
        fn $method_name:ident(
            &mut $self:ident
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty
    ) => {
        {
            #[allow(unused_unsafe, unused_variables, unused_assignments, unused_mut)]
            unsafe extern "C" fn method(
                this: *mut $crate::sys::godot_object,
                method_data: *mut $crate::libc::c_void,
                user_data: *mut $crate::libc::c_void,
                num_args: $crate::libc::c_int,
                args: *mut *mut $crate::sys::godot_variant
            ) -> $crate::sys::godot_variant {

                use std::cell::RefCell;
                use std::panic::{self, AssertUnwindSafe};

                let num_params = godot_wrap_method_parameter_count!($($pname,)*);

                if num_args != num_params {
                    godot_error!("Incorrect number of parameters: expected {} but got {}", num_params, num_args);
                    return $crate::Variant::new().to_sys();
                }


                let mut offset = 0;
                $(
                    let $pname = if let Some(val) = <$pty as $crate::GodotType>::from_sys_variant(&mut *(*args).offset(offset)) {
                        val
                    } else {
                        godot_error!("Incorrect argument type for argument {}", offset);
                        return $crate::Variant::new().to_sys();
                    };

                    offset += 1;
                )*

                let __rust_val = &*(user_data as *mut RefCell<$type_name>);
                let mut __rust_val = __rust_val.borrow_mut();

                let rust_ret = match panic::catch_unwind(AssertUnwindSafe(|| {
                    __rust_val.$method_name($($pname,)*);
                })) {
                    Ok(val) => val,
                    Err(err) => {
                        return $crate::Variant::new().to_sys();
                    }
                };

                <$retty as $crate::GodotType>::to_variant(&rust_ret).forget()
            }

            method
        }
    };
}
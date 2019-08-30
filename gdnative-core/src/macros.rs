#![macro_use]

/// Declare the API endpoint to initialize the gdnative API on startup.
///
/// By default this declares an extern function named `godot_gdnative_init`.
/// This can be overridden, for example:
///
/// ```ignore
/// // Declares an extern function named custom_gdnative_init instead of
/// // godot_gdnative_init.
/// godot_gdnative_init!(my_init_callback as custom_gdnative_init);
/// ```
///
/// Overriding the default entry point names can be useful if several gdnative
/// libraries are linked statically  to avoid name clashes.
#[macro_export]
macro_rules! godot_gdnative_init {
    () => {
        fn godot_gdnative_init_empty(_options: *mut $crate::sys::godot_gdnative_init_options) {}
        godot_gdnative_init!(godot_gdnative_init_empty);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_init_empty(_options: *mut $crate::sys::godot_gdnative_init_options) {}
        godot_gdnative_init!(godot_gdnative_init_empty as $fn_name);
    };
    ($callback:ident) => {
        godot_gdnative_init!($callback as godot_gdnative_init);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn $fn_name(options: *mut $crate::sys::godot_gdnative_init_options) {
            unsafe {
                $crate::GODOT_API = Some($crate::GodotApi::from_raw((*options).api_struct));
                $crate::GDNATIVE_LIBRARY_SYS = Some((*options).gd_native_library);
            }
            let api = $crate::get_api();
            // Force the initialization of the method table of common types. This way we can
            // assume that if the api object is alive we can fetch the method of these types
            // without checking for initialization.
            $crate::ReferenceMethodTable::get(api);

            $callback(options);
        }
    };
}

/// Declare the API endpoint invoked during shutdown.
///
/// By default this declares an extern function named `godot_gdnative_terminate`.
/// This can be overridden, for example:
///
/// ```ignore
/// // Declares an extern function named custom_gdnative_terminate instead of
/// // godot_gdnative_terminate.
/// godot_gdnative_terminate!(my_shutdown_callback as custom_gdnative_terminate);
/// ```
///
/// Overriding the default entry point names can be useful if several gdnative
/// libraries are linked statically  to avoid name clashes.
#[macro_export]
macro_rules! godot_gdnative_terminate {
    () => {
        fn godot_gdnative_terminate_empty(
            _options: *mut $crate::sys::godot_gdnative_terminate_options,
        ) {
        }
        godot_gdnative_terminate!(godot_gdnative_terminate_empty);
    };
    ($callback:ident) => {
        godot_gdnative_terminate!($callback as godot_gdnative_terminate);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_terminate_empty(
            _options: *mut $crate::sys::godot_gdnative_terminate_options,
        ) {
        }
        godot_gdnative_terminate!(godot_gdnative_terminate_empty as $fn_name);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn $fn_name(options: *mut $crate::sys::godot_gdnative_terminate_options) {
            $callback(options);

            unsafe {
                $crate::GODOT_API = None;
            }
        }
    };
}

/// Declare the API endpoint to initialize nativescript classes on startup.
///
/// By default this declares an extern function named `godot_nativescript_init`.
/// This can be overridden, for example:
///
/// ```ignore
/// // Declares an extern function named custom_nativescript_init instead of
/// // godot_nativescript_init.
/// godot_gdnative_terminate!(my_registration_callback as custom_nativescript_init);
/// ```
///
/// Overriding the default entry point names can be useful if several gdnative
/// libraries are linked statically  to avoid name clashes.
#[macro_export]
macro_rules! godot_nativescript_init {
    () => {
        fn godot_nativescript_init_empty(_init: $crate::init::InitHandle) {}
        godot_nativescript_init!(godot_nativescript_init_empty);
    };
    ($callback:ident) => {
        godot_nativescript_init!($callback as godot_nativescript_init);
    };
    (_ as $fn_name:ident) => {
        fn godot_nativescript_init_empty(_init: $crate::init::InitHandle) {}
        godot_nativescript_init!(godot_nativescript_init_empty as $fn_name);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        pub extern "C" fn $fn_name(handle: *mut $crate::libc::c_void) {
            unsafe {
                $callback($crate::init::InitHandle::new(handle));
            }
        }
    };
}

/// Print a message using the engine's logging system (visible in the editor).
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

/// Print a warning using the engine's logging system (visible in the editor).
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

/// Print an error using the engine's logging system (visible in the editor).
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
                unsafe { (get_api().$gd_method)(&mut self.0) }
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
                unsafe { (get_api().$gd_method)(&self.0, &other.0) }
            }
        }
    };

    (
        Eq for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            fn eq(&self, other: &Self) -> bool {
                unsafe { (get_api().$gd_method)(&self.0, &other.0) }
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

macro_rules! impl_guard_common_trait {
    (
        Drop for $Type:ident { $access:ident, $len:ident, } : $gd_method:ident
    ) => {
        impl<'a> Drop for $Type<'a> {
            fn drop(&mut self) {
                unsafe {
                    ($crate::get_api().$gd_method)(self.$access);
                }
            }
        }
    };

    (
        Clone for $Type:ident { $access:ident, $len:ident, } : $gd_method:ident
    ) => {
        impl<'a> Clone for $Type<'a> {
            fn clone(&self) -> Self {
                let $access = unsafe {
                    ($crate::get_api().$gd_method)(self.$access)
                };

                $Type {
                    $access,
                    $len: self.$len,
                    _marker: std::marker::PhantomData,
                }
            }
        }
    };
}

macro_rules! define_access_guard {
    (
        pub struct $Type:ident<'a>: $AccessPtr:ty {
            $access:ident = $access_gd_method:ident($BasePtr:ty),
            $len:ident = $len_gd_method:ident,
        }
        Guard<Target=$Target:ident> => $guard_gd_method:ident -> $OrigPtr:ty;
        $( $Trait:ident => $trait_gd_method:ident; )*
    ) => {
        pub struct $Type<'a> {
            $access: *mut $AccessPtr,
            $len: usize,
            _marker: std::marker::PhantomData<&'a $Target>,
        }

        impl<'a> $Type<'a> {
            unsafe fn new(arr: $BasePtr) -> Self {
                let $len = ($crate::get_api().$len_gd_method)(arr) as usize;
                let $access = ($crate::get_api().$access_gd_method)(arr);
                $Type {
                    $access,
                    $len,
                    _marker: std::marker::PhantomData,
                }
            }
        }

        unsafe impl<'a> $crate::access::Guard for $Type<'a> {
            type Target = $Target;
            fn len(&self) -> usize { self.$len }
            fn read_ptr(&self) -> *const Self::Target {
                unsafe {
                    let orig_ptr: $OrigPtr = ($crate::get_api().$guard_gd_method)(self.$access);
                    orig_ptr as *const Self::Target
                }
            }
        }

        $(
            impl_guard_common_trait!(
                $Trait for $Type { $access, $len, } : $trait_gd_method
            );
        )*
    };

    (
        pub struct $Type:ident<'a>: $AccessPtr:ty {
            $access:ident = $access_gd_method:ident($BasePtr:ty),
            $len:ident = $len_gd_method:ident,
        }
        Guard<Target=$Target:ident> + WritePtr => $guard_gd_method:ident -> $OrigPtr:ty;
        $( $Trait:ident => $trait_gd_method:ident; )*
    ) => {
        define_access_guard!(
            pub struct $Type<'a>: $AccessPtr {
                $access = $access_gd_method($BasePtr),
                $len = $len_gd_method,
            }
            Guard<Target=$Target> => $guard_gd_method -> $OrigPtr;
            $( $Trait => $trait_gd_method; )*
        );

        unsafe impl<'a> $crate::access::WritePtr for $Type<'a> { }
    };
}

macro_rules! godot_test {
    ($($test_name:ident $body:block)*) => {
        $(
            #[cfg(feature = "gd_test")]
            #[doc(hidden)]
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

/// Convenience macro to wrap an object's constructor into a function pointer
/// that can be passed to the engine when registering a class.
#[macro_export]
macro_rules! godot_wrap_constructor {
    ($_name:ty, $c:expr) => {{
        unsafe extern "C" fn constructor(
            this: *mut $crate::sys::godot_object,
            _method_data: *mut $crate::libc::c_void,
        ) -> *mut $crate::libc::c_void {
            // let val = $c($crate::NativeInstanceHeader{ this: this });
            let val = $c();

            let wrapper = <$_name as $crate::NativeClass>::UserData::new(val);
            wrapper.into_user_data() as *mut $crate::libc::c_void
        }

        constructor
    }};
}

/// Convenience macro to wrap an object's destructor into a function pointer
/// that can be passed to the engine when registering a class.
#[macro_export]
macro_rules! godot_wrap_destructor {
    ($name:ty) => {{
        #[allow(unused_unsafe)]
        unsafe extern "C" fn destructor(
            _this: *mut $crate::sys::godot_object,
            _method_data: *mut $crate::libc::c_void,
            user_data: *mut $crate::libc::c_void,
        ) -> () {
            let wrapper = <$_name as $crate::NativeClass>::UserData::consume_user_data_unchecked(user_data);
            drop(wrapper)
        }

        destructor
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! godot_wrap_method_parameter_count {
    () => {
        0
    };
    ($name:ident, $($other:ident,)*) => {
        1 + godot_wrap_method_parameter_count!($($other,)*)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! godot_wrap_method_inner {
    (
        $type_name:ty,
        $map_method:ident,
        fn $method_name:ident(
            $self:ident,
            $owner:ident : $owner_ty:ty
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

                use std::panic::{self, AssertUnwindSafe};
                use $crate::Instance;

                let __instance: Instance<$type_name> = Instance::from_raw(this, user_data);

                let num_params = godot_wrap_method_parameter_count!($($pname,)*);
                if num_args != num_params {
                    godot_error!("Incorrect number of parameters: expected {} but got {}", num_params, num_args);
                    return $crate::Variant::new().to_sys();
                }

                let mut offset = 0;
                $(
                    let _variant: &$crate::Variant = ::std::mem::transmute(&mut **(args.offset(offset)));
                    let $pname = if let Some(val) = <$pty as $crate::FromVariant>::from_variant(_variant) {
                        val
                    } else {
                        godot_error!("Incorrect argument type {:?} for argument {}",
                            _variant.get_type(),
                            offset);
                        return $crate::Variant::new().to_sys();
                    };

                    offset += 1;
                )*

                let rust_ret = match panic::catch_unwind(AssertUnwindSafe(move || {
                    let ret = __instance.$map_method(|__rust_val, $owner| __rust_val.$method_name($owner, $($pname,)*));
                    std::mem::drop(__instance);
                    ret
                })) {
                    Ok(val) => val,
                    Err(err) => {
                        return $crate::Variant::new().to_sys();
                    }
                };

                match rust_ret {
                    Ok(val) => <$retty as $crate::ToVariant>::to_variant(&val).forget(),
                    Err(err) => {
                        godot_error!("gdnative-core: method call failed with error: {:?}", err);
                        $crate::Variant::new().to_sys()
                    } 
                }
            }

            method
        }
    };
}

/// Convenience macro to wrap an object's method into a function pointer
/// that can be passed to the engine when registering a class.
#[macro_export]
macro_rules! godot_wrap_method {
    // mutable
    (
        $type_name:ty,
        fn $method_name:ident(
            &mut $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty
    ) => {
        godot_wrap_method_inner!(
            $type_name,
            map_mut_aliased,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
            ) -> $retty
        )
    };
    // immutable
    (
        $type_name:ty,
        fn $method_name:ident(
            & $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty
    ) => {
        godot_wrap_method_inner!(
            $type_name,
            map_aliased,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
            ) -> $retty
        )
    };
    // mutable without return type
    (
        $type_name:ty,
        fn $method_name:ident(
            &mut $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
        )
    ) => {
        godot_wrap_method!(
            $type_name,
            fn $method_name(
                &mut $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
            ) -> ()
        )
    };
    // immutable without return type
    (
        $type_name:ty,
        fn $method_name:ident(
            & $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
        )
    ) => {
        godot_wrap_method!(
            $type_name,
            fn $method_name(
                & $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
            ) -> ()
        )
    };
}

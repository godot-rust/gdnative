#![macro_use]

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
        fn godot_nativescript_init_empty(_init: $crate::nativescript::init::InitHandle) {}
        $crate::godot_nativescript_init!(godot_nativescript_init_empty);
    };
    ($callback:ident) => {
        $crate::godot_nativescript_init!($callback as godot_nativescript_init);
    };
    (_ as $fn_name:ident) => {
        fn godot_nativescript_init_empty(_init: $crate::nativescript::init::InitHandle) {}
        $crate::godot_nativescript_init!(godot_nativescript_init_empty as $fn_name);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        #[allow(unused_unsafe)]
        pub unsafe extern "C" fn $fn_name(handle: *mut $crate::libc::c_void) {
            if !$crate::private::is_api_bound() {
                return;
            }

            let __result = ::std::panic::catch_unwind(|| {
                $callback($crate::nativescript::init::InitHandle::new(handle));
            });

            if __result.is_err() {
                $crate::godot_error!("gdnative-core: nativescript_init callback panicked");
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! godot_wrap_method_parameter_count {
    () => {
        0
    };
    ($name:ident, $($other:ident,)*) => {
        1 + $crate::godot_wrap_method_parameter_count!($($other,)*)
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
            $(, #[opt] $opt_pname:ident : $opt_pty:ty)*
        ) -> $retty:ty
    ) => {
        {
            #[allow(unused_unsafe, unused_variables, unused_assignments, unused_mut)]
            #[allow(clippy::transmute_ptr_to_ptr)]
            unsafe extern "C" fn method(
                this: *mut $crate::sys::godot_object,
                method_data: *mut $crate::libc::c_void,
                user_data: *mut $crate::libc::c_void,
                num_args: $crate::libc::c_int,
                args: *mut *mut $crate::sys::godot_variant
            ) -> $crate::sys::godot_variant {

                use std::panic::{self, AssertUnwindSafe};
                use $crate::nativescript::{NativeClass, Instance, RefInstance};
                use $crate::object::{GodotObject, Ref};

                if user_data.is_null() {
                    $crate::godot_error!(
                        "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
                        stringify!($type_name),
                    );
                    return $crate::Variant::new().forget();
                }

                let this = match std::ptr::NonNull::new(this) {
                    Some(this) => this,
                    None => {
                        $crate::godot_error!(
                            "gdnative-core: base object pointer for {} is null (probably a bug in Godot)",
                            stringify!($type_name),
                        );
                        return $crate::Variant::new().forget();
                    },
                };

                let __catch_result = panic::catch_unwind(move || {
                    let this = <Ref<<$type_name as NativeClass>::Base, $crate::thread_access::Shared>>::from_sys(this);
                    let this = <<$type_name as NativeClass>::Base as GodotObject>::cast_ref(this.as_raw_unchecked());
                    let __instance = RefInstance::<$type_name>::from_raw_unchecked(this, user_data);

                    let num_args = num_args as isize;

                    let num_required_params = $crate::godot_wrap_method_parameter_count!($($pname,)*);
                    if num_args < num_required_params {
                        $crate::godot_error!("Incorrect number of parameters: required {} but got {}", num_required_params, num_args);
                        return $crate::Variant::new();
                    }

                    let num_optional_params = $crate::godot_wrap_method_parameter_count!($($opt_pname,)*);
                    let num_max_params = num_required_params + num_optional_params;
                    if num_args > num_max_params {
                        $crate::godot_error!("Incorrect number of parameters: expected at most {} but got {}", num_max_params, num_args);
                        return $crate::Variant::new();
                    }

                    let mut offset = 0;
                    $(
                        let _variant: &$crate::Variant = ::std::mem::transmute(&mut **(args.offset(offset)));
                        let $pname = match <$pty as $crate::FromVariant>::from_variant(_variant) {
                            Ok(val) => val,
                            Err(err) => {
                                $crate::godot_error!(
                                    "Cannot convert argument #{idx} ({name}) to {ty}: {err} (non-primitive types may impose structural checks)",
                                    idx = offset + 1,
                                    name = stringify!($pname),
                                    ty = stringify!($pty),
                                    err = err,
                                );
                                return $crate::Variant::new();
                            },
                        };

                        offset += 1;
                    )*

                    $(
                        let $opt_pname = if offset < num_args {
                            let _variant: &$crate::Variant = ::std::mem::transmute(&mut **(args.offset(offset)));

                            let $opt_pname = match <$opt_pty as $crate::FromVariant>::from_variant(_variant) {
                                Ok(val) => val,
                                Err(err) => {
                                    $crate::godot_error!(
                                        "Cannot convert argument #{idx} ({name}) to {ty}: {err} (non-primitive types may impose structural checks)",
                                        idx = offset + 1,
                                        name = stringify!($opt_pname),
                                        ty = stringify!($opt_pty),
                                        err = err,
                                    );
                                    return $crate::Variant::new();
                                },
                            };

                            offset += 1;

                            $opt_pname
                        }
                        else {
                            <$opt_pty as ::std::default::Default>::default()
                        };
                    )*

                    let __ret = __instance
                        .$map_method(|__rust_val, $owner| {
                            let ret = __rust_val.$method_name($owner, $($pname,)* $($opt_pname,)*);
                            <$retty as $crate::ToVariant>::to_variant(&ret)
                        })
                        .unwrap_or_else(|err| {
                            $crate::godot_error!("gdnative-core: method call failed with error: {:?}", err);
                            $crate::godot_error!("gdnative-core: check module level documentation on gdnative::user_data for more information");
                            $crate::Variant::new()
                        });

                    std::mem::drop(__instance);

                    __ret
                });

                __catch_result
                    .unwrap_or_else(|_err| {
                        $crate::godot_error!("gdnative-core: method panicked (check stderr for output)");
                        $crate::Variant::new()
                    })
                    .forget()
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
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        ) -> $retty:ty
    ) => {
        $crate::godot_wrap_method_inner!(
            $type_name,
            map_mut,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
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
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        ) -> $retty:ty
    ) => {
        $crate::godot_wrap_method_inner!(
            $type_name,
            map,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
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
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        )
    ) => {
        $crate::godot_wrap_method!(
            $type_name,
            fn $method_name(
                &mut $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
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
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        )
    ) => {
        $crate::godot_wrap_method!(
            $type_name,
            fn $method_name(
                & $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> ()
        )
    };
}

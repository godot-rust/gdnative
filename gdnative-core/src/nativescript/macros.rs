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

/// Declare all the API endpoints necessary to initialize a NativeScript library.
///
/// `godot_init!(init)` is a shorthand for:
///
/// ```ignore
/// godot_gdnative_init!();
/// godot_nativescript_init!(init);
/// godot_gdnative_terminate!();
/// ```
///
/// This declares three extern functions, named `godot_gdnative_init`,
/// `godot_nativescript_init`, and `godot_gdnative_terminate`. If you need different prefixes
/// to avoid name clashes when multiple GDNative libraries are linked statically, please use
/// the respective macros directly.
#[macro_export]
macro_rules! godot_init {
    ($callback:ident) => {
        $crate::godot_gdnative_init!();
        $crate::godot_nativescript_init!($callback);
        $crate::godot_gdnative_terminate!();
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
            #[derive(Copy, Clone, Default)]
            struct ThisMethod;

            use $crate::nativescript::{NativeClass, Instance, RefInstance, OwnerArg};
            use ::gdnative::FromVarargs;

            #[derive(FromVarargs)]
            struct Args {
                $($pname: $pty,)*
                $(#[opt] $opt_pname: $opt_pty,)*
            }

            #[allow(unused_variables, unused_assignments, unused_mut)]
            impl $crate::nativescript::init::method::StaticArgsMethod<$type_name> for ThisMethod {
                type Args = Args;
                fn call(
                    &self,
                    this: RefInstance<'_, $type_name, $crate::thread_access::Shared>,
                    Args { $($pname,)* $($opt_pname,)* }: Args,
                ) -> $crate::core_types::Variant {
                    this
                        .$map_method(|__rust_val, $owner| {
                            let ret = __rust_val.$method_name(
                                OwnerArg::from_safe_ref($owner),
                                $($pname,)*
                                $($opt_pname,)*
                            );
                            OwnedToVariant::owned_to_variant(ret)
                        })
                        .unwrap_or_else(|err| {
                            $crate::godot_error!("gdnative-core: method call failed with error: {}", err);
                            $crate::godot_error!("gdnative-core: check module level documentation on gdnative::user_data for more information");
                            $crate::core_types::Variant::new()
                        })
                }

                fn site() -> Option<$crate::log::Site<'static>> {
                    Some($crate::godot_site!($type_name::$method_name))
                }
            }

            $crate::nativescript::init::method::StaticArgs::new(ThisMethod)
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
    // owned
    (
        $type_name:ty,
        fn $method_name:ident(
            mut $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        ) -> $retty:ty
    ) => {
        $crate::godot_wrap_method_inner!(
            $type_name,
            map_owned,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> $retty
        )
    };
    // owned
    (
        $type_name:ty,
        fn $method_name:ident(
            $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        ) -> $retty:ty
    ) => {
        $crate::godot_wrap_method_inner!(
            $type_name,
            map_owned,
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
    // owned without return type
    (
        $type_name:ty,
        fn $method_name:ident(
            mut $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        )
    ) => {
        $crate::godot_wrap_method!(
            $type_name,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> ()
        )
    };
    // owned without return type
    (
        $type_name:ty,
        fn $method_name:ident(
            $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        )
    ) => {
        $crate::godot_wrap_method!(
            $type_name,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> ()
        )
    };
}

/// Convenience macro to create a profiling signature with a given tag.
///
/// The expanded code will panic at runtime if the file name or `tag` contains `::` or
/// any NUL-bytes.
///
/// See `nativescript::profiling::Signature` for more information.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// use gdnative_core::profile_sig;
/// use gdnative_core::nativescript::profiling::profile;
///
/// let answer = profile(profile_sig!("foo"), || 42);
/// assert_eq!(42, answer);
/// # }
/// ```
#[macro_export]
macro_rules! profile_sig {
    ($tag:expr) => {
        $crate::nativescript::profiling::Signature::new(file!(), line!(), $tag)
    };
}

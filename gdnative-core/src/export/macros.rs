#![macro_use]

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

            use $crate::export::{NativeClass, OwnerArg};
            use $crate::object::{Instance, RefInstance};
            use ::gdnative::derive::FromVarargs;

            #[derive(FromVarargs)]
            #[allow(clippy::used_underscore_binding)]
            struct Args {
                $($pname: $pty,)*
                $(#[opt] $opt_pname: $opt_pty,)*
            }

            #[allow(unused_variables, unused_assignments, unused_mut)]
            impl $crate::export::StaticArgsMethod<$type_name> for ThisMethod {
                type Args = Args;
                fn call(
                    &self,
                    this: RefInstance<'_, $type_name, $crate::object::ownership::Shared>,
                    Args { $($pname,)* $($opt_pname,)* }: Args,
                ) -> $crate::core_types::Variant {
                    this
                        .$map_method(|__rust_val, $owner| {
                            #[allow(unused_unsafe)]
                            unsafe {
                                let ret = __rust_val.$method_name(
                                    OwnerArg::from_safe_ref($owner),
                                    $($pname,)*
                                    $($opt_pname,)*
                                );
                                gdnative::core_types::OwnedToVariant::owned_to_variant(ret)
                            }
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

            $crate::export::StaticArgs::new(ThisMethod)
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

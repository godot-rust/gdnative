#![macro_use]

/// Legacy macro that declares all the API endpoints necessary to initialize a NativeScript library.
///
/// This is a shim for the new [`#[gdnative::init::callbacks]`][crate::init::callbacks] attribute
/// macro, which is now used to declare API callbacks along with the
/// [`GDNativeCallbacks`][crate::init::GDNativeCallbacks] trait.
#[macro_export]
#[deprecated = "use the #[gdnative::init::callbacks] attribute macro instead"]
macro_rules! godot_init {
    ($callback:ident) => {
        const _: () = {
            struct GDNativeCallbacksImpl;

            #[$crate::init::callbacks]
            unsafe impl $crate::init::GDNativeCallbacks for GDNativeCallbacksImpl {
                fn nativescript_init(handle: $crate::init::InitHandle) {
                    $callback(handle);
                }
            }
        };
    };
}

/// Legacy macro that declares all the API endpoints necessary to initialize a NativeScript library.
///
/// This is a shim for the new [`#[gdnative::init::callbacks]`][crate::init::callbacks] attribute
/// macro, which is now used to declare API callbacks along with the
/// [`GDNativeCallbacks`][crate::init::GDNativeCallbacks] trait.
#[macro_export]
#[deprecated = "use the #[gdnative::init::callbacks] attribute macro instead"]
macro_rules! godot_nativescript_init {
    () => {
        const _: () = {
            struct GDNativeCallbacksImpl;

            #[$crate::init::callbacks]
            unsafe impl $crate::init::GDNativeCallbacks for GDNativeCallbacksImpl {}
        };
    };
    ($callback:ident) => {
        const _: () = {
            struct GDNativeCallbacksImpl;

            #[$crate::init::callbacks]
            unsafe impl $crate::init::GDNativeCallbacks for GDNativeCallbacksImpl {
                fn nativescript_init(handle: $crate::init::InitHandle) {
                    $callback(handle);
                }
            }
        };
    };
    (_ as $fn_name:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro with a prefix instead");
    };
    ($callback:ident as $fn_name:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro with a prefix instead");
    };
}

/// This macro now does nothing. It is provided only for limited backwards compatibility.
///
/// Use the new [`#[gdnative::init::callbacks]`][crate::init::callbacks] attribute macro and,
/// [`GDNativeCallbacks`][crate::init::GDNativeCallbacks] trait instead.
#[macro_export]
#[deprecated = "use the #[gdnative::init::callbacks] attribute macro instead"]
macro_rules! godot_gdnative_init {
    () => {};
    ($callback:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro instead");
    };
    (_ as $fn_name:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro with a prefix instead");
    };
    ($callback:ident as $fn_name:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro with a prefix instead");
    };
}

/// This macro now does nothing. It is provided only for limited backwards compatibility.
///
/// Use the new [`#[gdnative::init::callbacks]`][crate::init::callbacks] attribute macro and,
/// [`GDNativeCallbacks`][crate::init::GDNativeCallbacks] trait instead.
#[macro_export]
#[deprecated = "use the #[gdnative::init::callbacks] attribute macro instead"]
macro_rules! godot_gdnative_terminate {
    () => {};
    ($callback:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro instead");
    };
    (_ as $fn_name:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro with a prefix instead");
    };
    ($callback:ident as $fn_name:ident) => {
        ::std::compile_error!("this syntax is no longer supported. use the #[gdnative::init::callbacks] attribute macro with a prefix instead");
    };
}

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

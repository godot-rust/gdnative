//! Global initialization and termination of the library.
//!
//! This module provides all the plumbing required for global initialization and shutdown of godot-rust.

mod info;
mod init_handle;
mod macros;
mod terminate_handle;

pub mod diagnostics;

/// Internal low-level API for use by macros and generated bindings. Not a part of the public API.
#[doc(hidden)]
pub mod private;

pub use info::*;
pub use init_handle::*;
pub use terminate_handle::*;

/// Trait for declaring library-level GDNative callbacks. See module-level docs for examples.
///
/// Each end-user library must contain one and exactly one implementation of this trait.
/// It must be annotated with the [`#[gdnative::init::callbacks]`][callbacks]
/// proc-macro attribute.
///
/// The most commonly useful callback to implement is [`Self::nativescript_init`], which allows
/// the implementor to manually register [`NativeClass`][crate::export::NativeClass] types so
/// they can be used from Godot.
///
/// An alternative to manual registration is the `inventory` feature, which automatically
/// registers derived [`NativeClass`][crate::export::NativeClass] on
/// [supported platforms][inventory-support], which at the time of writing includes every platform
/// that Godot officially supports except WASM.
///
/// For all callbacks in this trait, it's guaranteed that the Godot API is available when called.
///
/// ## Example
///
/// With manual registration:
///
/// ```no_run
/// use gdnative::prelude::*;
///
/// #[derive(NativeClass)]
/// # #[no_constructor]
/// struct HelloWorld { /* ... */ }
///
/// #[methods]
/// impl HelloWorld { /* ... */ }
///
/// struct MyLibrary;
///
/// #[gdnative::init::callbacks]
/// impl GDNativeCallbacks for MyLibrary {
///     fn nativescript_init(handle: InitHandle) {
///         handle.add_class::<HelloWorld>();
///     }
/// }
/// ```
///
/// With automatic registration:
///
/// ```no_run
/// use gdnative::prelude::*;
///
/// #[derive(NativeClass)]
/// # #[no_constructor]
/// struct HelloWorld { /* ... */ }
///
/// #[methods]
/// impl HelloWorld { /* ... */ }
///
/// struct MyLibrary;
///
/// #[gdnative::init::callbacks]
/// impl GDNativeCallbacks for MyLibrary {}
/// ```
///
/// [inventory-support]: https://github.com/dtolnay/inventory#how-it-works
/// ```
pub trait GDNativeCallbacks: private::TheGDNativeCallbacksAttributeIsRequired {
    /// Callback invoked on startup, before any other callbacks.
    ///
    /// At the time [`Self::gdnative_init`] is called, it is guaranteed that:
    ///
    /// - No API callbacks have been invoked before.
    #[inline]
    #[allow(unused)]
    fn gdnative_init(info: InitializeInfo) {}

    /// Callback invoked on shutdown, after all other callbacks.
    ///
    /// At the time [`Self::gdnative_terminate`] is called, it is guaranteed that:
    ///
    /// - No API callbacks will be invoked after.
    #[inline]
    #[allow(unused)]
    fn gdnative_terminate(info: TerminateInfo) {}

    /// Callback invoked after startup, immediately after `gdnative_init`, if the `singleton`
    /// option is set for the `GDNativeLibrary` resource.
    ///
    /// **Attention:** This is **NOT** what you're looking for! `gdnative_singleton` has nothing
    /// to do with exposing "singleton" objects or static-looking methods to GDScript. Instead,
    /// create a [`NativeClass`][crate::export::NativeClass] inheriting `Node` and make that an
    /// auto-load in your project settings. See [the FAQ][auto-load-faq] for an example.
    ///
    /// Despite the confusing name, what the `singleton` option does is to mark the GDNative
    /// library for automatic initialization at the startup of the application before any game
    /// content and scripts are loaded, which might be useful in some use cases. The extra
    /// callback itself doesn't have any special qualities.
    ///
    /// At the time [`Self::gdnative_singleton`] is called, it is guaranteed that:
    ///
    /// - [`Self::gdnative_init`] has been invoked exactly once, immediately before.
    /// - No other API callbacks have been invoked.
    ///
    /// It is NOT guaranteed that:
    ///
    /// - There are any practical uses to this callback. **You do not need this to create a
    ///   singleton object, nor does it help you.** Use an [auto-load singleton][auto-load-faq]
    ///   instead.
    ///
    /// [auto-load-faq]: https://godot-rust.github.io/book/gdnative/faq/code.html#can-i-implement-static-methods-in-gdnative
    #[inline]
    fn gdnative_singleton() {}

    /// Callback invoked after startup, before `NativeScript`s are registered.
    ///
    /// At the time [`Self::nativescript_init`] is called, it is guaranteed that:
    ///
    /// - [`Self::gdnative_init`] has been invoked exactly once.
    ///
    /// It is NOT guaranteed that:
    ///
    /// - This is immediately invoked after [`Self::gdnative_init`].
    /// - [`Self::nativescript_init`] has not been invoked before.
    #[inline]
    #[allow(unused)]
    fn nativescript_init(handle: InitHandle) {}

    /// Callback invoked during engine cleanup if NativeScript has been used and the library
    /// is still loaded.
    ///
    /// Despite the naming, this is not guaranteed to be paired with [`Self::nativescript_init`].
    /// Godot may terminate (and later re-initialize) the library without calling this function
    /// due to hot-reloading.
    ///
    /// While a handle argument is provided by the engine, its purpose is currently unclear.
    ///
    /// At the time [`Self::nativescript_terminate`] is called, it is guaranteed that:
    ///
    /// - [`Self::gdnative_init`] has been invoked exactly once.
    /// - [`Self::nativescript_init`] has been invoked at least once.
    ///
    /// It is NOT guaranteed that:
    ///
    /// - This will always be called before [`Self::gdnative_terminate`].
    #[inline]
    #[allow(unused)]
    fn nativescript_terminate(handle: TerminateHandle) {}

    /// Callback invoked every frame if any NativeScripts are being used.
    #[inline]
    fn nativescript_frame() {}

    /// Callback invoked before a thread managed by Godot other than the main thread is entered,
    /// if any NativeScripts are being used.
    #[inline]
    fn nativescript_thread_enter() {}

    /// Callback invoked after a thread managed by Godot other than the main thread has been
    /// exited from, if any NativeScripts are being used.
    #[inline]
    fn nativescript_thread_exit() {}
}

bitflags::bitflags! {
    /// Initialization level used to distinguish the source of init actions, such as class registration.
    /// Internal API.
    #[doc(hidden)]
    pub struct InitLevel: u8 {
        /// Init level for automatic registration
        const AUTO = 1;
        /// Init level for user code
        const USER = 2;
    }
}

#[doc(hidden)]
#[cfg(feature = "inventory")]
#[inline]
pub fn auto_register(init_handle: InitHandle) {
    for plugin in inventory::iter::<crate::private::AutoInitPlugin> {
        (plugin.f)(init_handle);
    }
}

#[doc(hidden)]
#[cfg(not(feature = "inventory"))]
#[inline]
pub fn auto_register(_init_handle: InitHandle) {
    // Nothing to do here.
}

#[allow(deprecated)]
pub use crate::{
    godot_gdnative_init, godot_gdnative_terminate, godot_init, godot_nativescript_init,
};

#[doc(inline)]
pub use gdnative_derive::callbacks;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod methods;
mod native_script;
mod profiled;
mod variant;

#[proc_macro_attribute]
pub fn methods(meta: TokenStream, input: TokenStream) -> TokenStream {
    methods::derive_methods(meta, input)
}

/// Makes a function profiled in Godot's built-in profiler. This macro automatically
/// creates a tag using the name of the current module and the function by default.
///
/// This attribute may also be used on non-exported functions. If the GDNative API isn't
/// initialized when the function is called, the data will be ignored silently.
///
/// A custom tag can also be provided using the `tag` option.
///
/// See the `gdnative::nativescript::profiling` for a lower-level API to the profiler with
/// more control.
///
/// # Examples
///
/// ```ignore
/// mod foo {
///     // This function will show up as `foo/bar` under Script Functions.
///     #[gdnative::profiled]
///     fn bar() {
///         std::thread::sleep(std::time::Duration::from_millis(1));
///     }
/// }
/// ```
///
/// ```ignore
/// // This function will show up as `my_custom_tag` under Script Functions.
/// #[gdnative::profiled(tag = "my_custom_tag")]
/// fn baz() {
///     std::thread::sleep(std::time::Duration::from_millis(1));
/// }
/// ```
#[proc_macro_attribute]
pub fn profiled(meta: TokenStream, input: TokenStream) -> TokenStream {
    profiled::derive_profiled(meta, input)
}

#[proc_macro_derive(
    NativeClass,
    attributes(inherit, export, opt, user_data, property, register_with)
)]
pub fn derive_native_class(input: TokenStream) -> TokenStream {
    native_script::derive_native_class(input)
}

#[proc_macro_derive(ToVariant, attributes(variant))]
pub fn derive_to_variant(input: TokenStream) -> TokenStream {
    variant::derive_to_variant(variant::ToVariantTrait::ToVariant, input)
}

#[proc_macro_derive(OwnedToVariant, attributes(variant))]
pub fn derive_owned_to_variant(input: TokenStream) -> TokenStream {
    variant::derive_to_variant(variant::ToVariantTrait::OwnedToVariant, input)
}

#[proc_macro_derive(FromVariant, attributes(variant))]
pub fn derive_from_variant(input: TokenStream) -> TokenStream {
    variant::derive_from_variant(input)
}

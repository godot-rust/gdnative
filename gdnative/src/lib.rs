//! # Rust bindings for the Godot game engine
//!
//! This crate contains high-level wrappers around the Godot game engine's GDNative API.
//! Some of the types were automatically generated from the engine's JSON API description,
//! and some other types are hand made wrappers around the core C types.
//!
//! ## Core types
//!
//! Wrappers for most core types expose safe Rust interfaces, and it's unnecessary
//! to mind memory management most of the times. The exceptions are
//! [`VariantArray`](core_types::VariantArray) and [`Dictionary`](core_types::Dictionary),
//! internally reference-counted collections with _interior mutability_ in Rust parlance.
//! These types are modelled using the _typestate_ pattern to enforce that the official
//! [thread-safety guidelines][thread-safety]. For more information, read the type-level
//! documentation for these types.
//!
//! Since it is easy to expect containers and other types to allocate a copy of their
//! content when using the `Clone` trait, some types do not implement `Clone` and instead
//! implement [`NewRef`](object::NewRef) which provides a `new_ref(&self) -> Self` method
//! to create references to the same collection or object.
//!
//! ## Generated API types
//!
//! The [`api`] module contains high-level wrappers for all the API types generated from a
//! JSON description of the API. The generated types are tied to a specific version, typically
//! the latest Godot 3.x release (at the time of the godot-rust release).
//! If you want to use the bindings with another version of the engine, read the notes on
//! the `custom-godot` feature flag below.
//!
//! ### Memory management
//!
//! API types may be reference-counted or manually-managed. This is indicated by the
//! `RefCounted` and `ManuallyManaged` marker traits.
//!
//! The API types can exist in three reference forms: bare, [`TRef`](object::TRef) and [`Ref`](object::Ref).
//! Bare references to API types, like `&'a Node`, represent valid and safe references to Godot objects.
//! As such, API methods may be called safely on them. `TRef` adds typestate tracking, which
//! enable additional abilities like being able to be passed to the engine. `Ref`, or
//! _persistent_ references, have `'static` lifetime, but are not always safe to use. For more
//! information on how to use persistent references safely, see the [`object`] module documentation
//! or the corresponding [book chapter][gdnative-overview].
//!
//! ## Feature flags
//! All features are disabled by default.
//!
//! Functionality toggles:
//!
//! * **`async`**<br>
//!   Activates async functionality, see [`tasks`] module for details.
//!
//! * **`serde`**<br>
//!   Enable for `serde` support of several core types. See also [`Variant`](core_types::Variant).
//!
//! * **`inventory`**<br>
//!   Enables automatic class registration via `inventory`.
//!
//!   **Attention:** Automatic registration is unsupported on some platforms, notably WASM. `inventory`
//!   can still be used for iterative development if such platforms are targeted, in which case the
//!   run-time diagnostic [`init::diagnostics::missing_manual_registration`] may be helpful.
//!
//!   Please refer to [the `rust-ctor` README][ctor-repo] for an up-to-date listing of platforms
//!   that *do* support automatic registration.
//!
//! Bindings generation:
//!
//! * **`custom-godot`**<br>
//!   When active, tries to locate a Godot executable on your system, in this order:
//!   1. If a `GODOT_BIN` environment variable is defined, it will interpret it as a path to the binary
//!      (not directory).
//!   2. An executable called `godot`, accessible in your system's PATH, is used.
//!   3. If neither of the above is found, an error is generated.
//!
//!   The symbols in [`api`] will be generated in a way compatible with that engine version.
//!   This allows to use Godot versions older than the currently supported ones.
//!
//!   See [Custom Godot builds][custom-godot] for detailed instructions.
//!
//! * **`formatted`**<br>
//!   Enable if the generated binding source code should be human-readable and split
//!   into multiple files. This can also help IDEs that struggle with a single huge file.
//!
//! * **`ptrcall`**<br>
//!   Enables the `ptrcall` convention for calling Godot API methods. This increases performance, at the
//!   cost of forward binary compatibility with the engine. Binaries built with `ptrcall` enabled
//!   **may exhibit undefined behavior** when loaded by a different version of Godot, even when there are
//!   no breaking API changes as far as GDScript is concerned. Notably, the addition of new default
//!   parameters breaks any code using `ptrcall`.
//!
//!   Cargo features are additive, and as such, it's only necessary to enable this feature for the final
//!   `cdylib` crates, whenever desired.
//!
//! [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
//! [gdnative-overview]: https://godot-rust.github.io/book/gdnative-overview.html
//! [custom-godot]: https://godot-rust.github.io/book/advanced-guides/custom-godot.html
//! [ctor-repo]: https://github.com/mmastrac/rust-ctor
//!
//!

#![doc(html_logo_url = "https://github.com/godot-rust/gdnative/raw/master/assets/godot-ferris.svg")]

// Workaround (rustdoc 1.55):
// Items, which are #[doc(hidden)] in their original crate and re-exported with a wildcard, lose
// their hidden status. Re-exporting them manually and hiding the wildcard solves this.
#[doc(inline)]
pub use gdnative_core::{
    core_types, derive, export, godot_dbg, godot_error, godot_print, godot_site, init, log, object,
    profiler,
};

pub mod globalscope;

// Implementation details (e.g. used by macros).
// However, do not re-export macros (on crate level), thus no wildcard
#[doc(hidden)]
pub use gdnative_core::{libc, private, sys};

/// Curated re-exports of common items.
pub mod prelude;

/// Bindings for the Godot Class API.
#[doc(inline)]
pub use gdnative_bindings as api;

#[doc(inline)]
#[cfg(feature = "async")]
pub use gdnative_async as tasks;

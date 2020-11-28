# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **The minimum compatible engine version is now 3.2-stable.**

## [0.9.1] - 2020-10-19

### Added

- Support for RPC modes using the `export` attribute, e.g. `#[export(rpc = "remote_sync")]`.

- Added the convenience method `Vector2Godot::clamped`.

- Added Godot-equivalent methods for `Plane`.

### Fixed

- Fixed a problem where incorrect documentation may be generated when building from case-insensitive file systems.

- Fixed a case of undefined behavior when `Instance::new` is called for non-tool scripts in the editor.

- Fixed a type mismatch problem that may prevent compilation on some target platforms.

- Fixed potential compilation error in case of `TypeId` widening.

## [0.9.0] - 2020-09-20

### Added

- All public functions now have the `#[inline]` attribute, improving cross-crate inlining in builds without LTO.

- A curated `prelude` module is added in `gdnative` crate, containing common types and traits.

- Added the `SubClass` trait, which allows for static up-casts and static validation of downcasts.

- Added the `OwnedToVariant` trait and derive macro which enabled `Variant` conversion for more types.

- The `NativeScript` and `#[methods]` proc-macros now report errors more accurately.

- Added the `godot_init` convenience macro that declares all three endpoints for common use cases.

- Added more extension methods for `Vector2`, `Vector3` and `Color`.

- Added wrappers for `GodotString::get_basename` and `get_extensions`.

- Added a high-level interface to the Godot script profiler in the `gdnative::nativescript::profiling` module, and in the `#[gdnative::profiled]` attribute.

- Added before/after hooks for the `#[property]` attribute.

- API methods now have generated documentation according to Godot documentation XMLs. The Godot docs contain custom markup which isn't currently parsed. We expect to improve the generated docs in the following releases.

- Added custom resource example.

### Changed

- **The default API version is now Godot 3.2.3-stable.**

- The object reference system is revamped using the typestate pattern, with semantics that model Godot behavior more accurately, allowing for clearer boundaries between safe and unsafe code.

- API methods are now generic over types that can be converted to `Variant`, `GodotString`, or generated API types, removing the need for boilerplate code like `&thing.into()`.

- Enums in the API are now represented more accurately as newtypes around `i64`. They are available in the same modules where their associated objects are defined.

- `Dictionary` and `VariantArray` now use the typestate pattern as well.

- The typed arrays are unified into a generic `TypedArray` type. The old names remain as type aliases.

- Moved generated bindings into the `gdnative::api` module, making the top-level namespace easier to navigate.

- It's now possible to crate custom binding crates without forking the repository using the generator, since `gdnative_bindings_generator::Api::new` now takes JSON input as an argument.

- Separated core wrappers and NativeScript support into the `core_types` and `nativescript` modules.

- Cleaned up the public interface with regards to intended usage. The public API no longer uses `gdnative-sys` types.

- The `new_ref` method is now in a `NewRef` trait.

- High-level wrappers are added for `godot_gdnative_init` and `godot_gdnative_terminate` callback arguments.

- Improved source links on docs.rs.

- `bindgen` is updated to 0.55.1.

- `euclid` is updated to 0.22.1.

- Improved build time performance.

### Removed

- Removed deprecated items from the public interface in 0.8.1.

- Removed `gdnative-sys` types from the public interface. `gdnative-sys` is considered an internal dependency starting from 0.9.

- Removed the `Object` and `Reference` wrappers from `gdnative-core`. The same types are available in `gdnative-bindings`.

- Removed generated bindings for virtual methods, since they cannot actually be called.

- Removed `From` implementations for `Variant` since `ToVariant` is much more comprehensive.

### Fixed

- Fixed typos in variant names of `VariantOperator` and `GodotError`.

- `StringName::from_str` now returns `Self` correctly.

- Fixed a case of undefined behavior that may manifest as crashes when some specific methods that return `VariantArray` are called.

- Fixed an issue with platform headers when building on Windows with the `gnu` toolchain that prevented compilation.

- Macros can now be used with qualified imports, removing the need for `#[macro_use]`.

- Fixed an issue where `Rid` arguments passed to API methods are incorrect due to use-after-free.

## [0.8.1] - 2020-05-31

### Added

- Exported methods can now have optional arguments. Arguments with the `#[opt]` attribute are optional in the scripting API. Default values are obtained using `Default` if not provided by the caller.

- Added a `claim` method on the `GodotObject` trait, which clones the reference if the underlying type is reference-counted (extends from `Reference`), or aliases it if it isn't. This replaces the `Class::from_sys(object.to_sys())` "idiom" that relied on hidden public items (`from_sys` and `to_sys`) that are not actually intended to be part of the public API.

- Fields can now be skipped with the `#[variant(skip)]` attribute when deriving `FromVariant` and `ToVariant`.

- Implemented various methods on `Basis`.

- Implemented `Display` for `GodotString`.

- Implemented `Debug` for core typed arrays.

- Added `Aether<T>`, a special `UserData` wrapper for ZSTs. This type does not perform any allocation or synchronization at runtime, but produces a value using `Default` each time it's mapped.

- Include paths for the Android SDK can now be inferred from environment variables.

- A dodge-the-creeps example has been added to the repo.

### Changed

- Initialization errors are now reported using GDNative APIs, instead of panics. This includes situations where API struct versions mismatch, or if some API functions are unavailable.

- Paths to SDKs for Apple platforms are now obtained using `xcrun`.

- The public fields of `ExportInfo` are now deprecated. They will become private in 0.9. Use one of the constructors or `Export::export_info` instead.

- Several public types that are unused in the current API have been deprecated.

- Free-on-drop wrappers are now deprecated. Use of free-on-drop wrappers is no longer recommended due to upcoming changes in ownership semantics in 0.9. Users are suggested to call `free` or `queue_free` manually instead. They will be removed in 0.9.

### Fixed

- Fixed a problem where the build script for `gdnative-sys` will try to include macOS headers when building for mobile targets from a Mac, causing the build to fail.

- Fixed SDK include paths for the iOS Simulator, whose SDK was separate from the one for real iOS devices. This allows building for the iOS Simulator platform.

- Fixed a case of undefined behavior (UB) when Rust scripts are attached to incompatible base classes in the Godot Editor (e.g. attaching a `NativeClass` with `Base = Node2D` to a `Spatial` node).

## [0.8.0] - 2020-03-09

### Added

- Field attribute `property` on derived `NativeClass` types, which can be used to quickly export simple properties.

- The behavior of derived `FromVariant` and `ToVariant` implementations can be customized with the `variant` field attribute.

- New example projects.

- A `godot_dbg!` macro for quick and dirty debugging that works like the standard `dbg!`, but prints to the Godot debug console.

### Changed

- **The default API version is now Godot 3.2-stable.**

- The `FromVariant` trait now reports detailed information on failure.

- The API for property registration is reworked to provide better ergonomics and static type checking for editor hints.

- `LocalCellData` is now the default user-data wrapper type. This wrapper allows non-`Send` types to be used as `NativeClass`es, and produces a runtime error whenever a value is accessed from a different thread than where it was created.

### Removed

- Removed the old-style `godot_class!` macro.

### Fixed

- Fixed an `unused_parens` warning when using the `NativeClass` derive macro.

- Fixed handling of unknown enums with duplicate values, which prevented code generation for Godot version `3.2`.

- Fixed a memory leak where a `Drop` implementation wasn't generated for non-instanciable reference-counted types.

- Fixed a memory leak where reference-counted types get an extra reference count when returned from the engine.

- Fixed bindings generation when building for iOS using `cargo-lipo`.

## [0.7.0] - 2019-12-22

### Added

- Procedural-macro `methods` which can be applied to `impl` blocks and allows
  a more natural Rust syntax for creating script types and exporting functions
  using attributes.

- The `ToVariant` and `FromVariant` traits, including derive macros.
  These traits can be used to define how custom types can be constructed from
  `Variant`s or be extracted from existing `Variant`s.
  Any type implementing `FromVariant` can be used as a parameter to an exported
  function. Any type implementing `ToVariant` can be returned from exported
  functions.

- Derive-macro for `NativeClass` trait.

- Every type implementing `NativeClass` provides a "user data" storage type
  which is used to control how the script data is stored internally. A default
  value is provided via the procedural macro.

- Iterators for Godot collection types.

- `Instance<T>` type which contains the Godot owner object and the script data
  (implements `ToVariant` and `FromVariant`, so it can be used as a parameter
  or return type).

- Generated class wrappers now include associated constants for constants
  provided by Godot.

- New example projects.

### Changed

- The code now uses the Rust 2018 edition.

- The API description of Godot classes was updated to the stable Godot version
  `3.1.1`.

- The GDNative API description was updated to the stable Godot version `3.1.1`
  and now includes added GDNative extensions.

- The `NativeClass` trait has changed a lot and was split into two traits to
  allow the `methods` procedural macro to generate a description of exported
  methods.

- The generated class bindings are stored in a single crate again and use the
  `Deref` trait to implement inheritance.

### Removed

- The "domain-grouped" crates for generated bindings are merged into a single
  crate, so the individual crates are no longer in use.

### Fixed

- Fixed a memory safety issue where the strings used to register signals are
  dropped before the API call.

- Fixed a correctness issue where the layout of method arguments is
  incorrectly assumed to be continuous, causing invalid memory access when
  calling methods with multiple arguments.

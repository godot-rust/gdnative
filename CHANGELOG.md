# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - TBD

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

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.3] - 2023-01-30

This is a backwards-compatible release; thus no removals or breaking changes.

### Added

- RPC modes are now supported for properties: `#[property(rpc = "mode")]`. (#1006)

### Changed

- Specialized pool array aliases such as `PoolByteArray` are now deprecated and will be removed in the next version. Use `PoolArray<u8>` instead. (#1007)

### Fixed

- Types with one-way conversions can now be exposed through the `#[property]` attribute. (#1013)
- `FromVariant` for marker types should no longer fail in the fringe case where null object variants are explicitly used. (#1012)

## [0.11.2] - 2023-01-09

This is a hot-fix release for a high priority issue.

### Fixed

- API methods that may return null object pointers should no longer panic. ([#1002](https://github.com/godot-rust/gdnative/pull/1002))

## [0.11.1] - 2023-01-06

This is a backwards-compatible release; thus no removals or breaking changes.

### Added

- `NativeClass` can now be derived for generic types. Additionally, `#[monomorphize]` can be used to name concrete monomorphizations. ([#983](https://github.com/godot-rust/gdnative/pull/983))
- With the optional `inventory` feature enabled, `NativeClass`es and their `#[monomorphize]`d aliases can now be automatically registered on supported platforms. ([#999](https://github.com/godot-rust/gdnative/pull/999))
- `#[methods]` now supports async-await coroutines. ([#975](https://github.com/godot-rust/gdnative/pull/975))
- Mix-in impl blocks can now be created through `#[methods(mixin = "Name")]`. These blocks have a many-to-many relationship with `NativeClass`es, and can be generic, or trait implementations. ([#999](https://github.com/godot-rust/gdnative/pull/999))
- Added a Third-Person-Shooter example. ([#977](https://github.com/godot-rust/gdnative/pull/977))
- Variant derive macros now support stringly and numeric representations for fieldless enums. ([#964](https://github.com/godot-rust/gdnative/pull/964))
- `FromVariant` can now be derived for uninhabitable enums. ([#962](https://github.com/godot-rust/gdnative/pull/962))
- Dedicated accessor methods are now generated for indexed properties, such as `SpatialMaterial::albedo_texture`. ([#970](https://github.com/godot-rust/gdnative/pull/970))
- Implemented additional geometric operations on `Transform3D`. ([#898](https://github.com/godot-rust/gdnative/pull/898))
- Android targets are now supported on macOS running on Apple Silicon.  ([#982](https://github.com/godot-rust/gdnative/pull/982))

### Changed

- Improved panic messages in init/terminate callbacks. ([#960](https://github.com/godot-rust/gdnative/pull/960))
- `ptrcall`s are now opt-in, with the `ptrcall` feature flag. This improves binary compatibility in the default configuration. ([#973](https://github.com/godot-rust/gdnative/pull/973))

### Fixed

- Variant derive macros now work properly with generic types with bounds. ([#961](https://github.com/godot-rust/gdnative/pull/961))
- `Transform::interpolate_with` now has behavior consistent with Godot 3 (spherical interpolation). ([#998](https://github.com/godot-rust/gdnative/pull/998))
- The correct number of arguments are now reported when an invalid argument list is provided for a method with optional arguments. ([#1000](https://github.com/godot-rust/gdnative/pull/1000))

## [0.11.0] - 2022-10-02

### Changed

- Changed supported Godot version to 3.5.1 ([#910](https://github.com/godot-rust/godot-rust/pull/910))
- MSRV is now 1.63 ([#910](https://github.com/godot-rust/godot-rust/pull/910))
- Prefixed `NativeClass` methods for manual implementors ([#955](https://github.com/godot-rust/godot-rust/pull/955))

### Fixed

- `godot_init` may not find some symbols ([#954](https://github.com/godot-rust/godot-rust/pull/954))

### Removed

- `Transform2D::from_rotation_translation_scale()` constructor ([#910](https://github.com/godot-rust/godot-rust/pull/910))
- `RefInstance` and `TypedArray` type aliases ([#955](https://github.com/godot-rust/godot-rust/pull/955))


## [0.10.2] - 2022-10-02

Last maintenance release for Godot 3.4.

### Added

- `globalscope::load()` function ([#940](https://github.com/godot-rust/godot-rust/pull/940), [#941](https://github.com/godot-rust/godot-rust/pull/941))
- `Color` constructors from HTML string and integers ([#939](https://github.com/godot-rust/godot-rust/pull/939))
- Version check to warn if Godot is not 3.4 ([#942](https://github.com/godot-rust/godot-rust/pull/942))
- Support for iOS simulator on Mac M1 ([#944](https://github.com/godot-rust/godot-rust/pull/944))

### Fixed

- During tests, `get_api()` no longer aborts  ([#929](https://github.com/godot-rust/godot-rust/pull/929))
- Confusing `Transform2D` constructor ([#930](https://github.com/godot-rust/godot-rust/pull/930))
- Bug in `Rect2::intersects()` ([#948](https://github.com/godot-rust/godot-rust/pull/948))
- Bug in `Vector2::rotated()` ([#952](https://github.com/godot-rust/godot-rust/pull/952))


## [0.10.1] - 2022-09-03

This is a backwards-compatible release; thus no removals or breaking changes.

### Added

- New export API, allowing to omit owner ([#872](https://github.com/godot-rust/godot-rust/pull/872), [#933](https://github.com/godot-rust/godot-rust/pull/933))
- Export and Variant conversion for `Vec`/`HashMap`/`HashSet` ([#883](https://github.com/godot-rust/godot-rust/pull/883))
- Attribute `deref_return` to return reference-like objects ([#870](https://github.com/godot-rust/godot-rust/pull/870))
- Classes `Rect2` and `Aabb` now have methods ([#867](https://github.com/godot-rust/godot-rust/pull/867))
- Module `globalscope` with GDScript utility functions, e.g. `lerp`, `smoothstep` ([#901](https://github.com/godot-rust/godot-rust/pull/901), [#906](https://github.com/godot-rust/godot-rust/pull/906))
- `Varargs` has new API for length checks, type conversions and errors ([#892](https://github.com/godot-rust/godot-rust/pull/892))
- Method `Axis::to_unit_vector()` ([#867](https://github.com/godot-rust/godot-rust/pull/867))

### Fixed

- `StringName` traits `Eq` and `Ord` had a bug in GDNative API ([#912](https://github.com/godot-rust/godot-rust/pull/912))
- `register_properties` naming collision ([#888](https://github.com/godot-rust/godot-rust/pull/888))
- Outdated GDNative API checks prevented compilation of Godot 3.5 RC ([#909](https://github.com/godot-rust/godot-rust/pull/909))
- Android: allow usage of new NDK paths ([#754](https://github.com/godot-rust/godot-rust/pull/754))
- Use `ManuallyDrop` in ptrcalls to prevent drop reordering ([#924](https://github.com/godot-rust/godot-rust/pull/924))
- Fix memory leaks in `as_arg` tests ([#925](https://github.com/godot-rust/godot-rust/pull/925))
- `VariantArray` iterator skip ([#936](https://github.com/godot-rust/godot-rust/pull/936))
- Proc-macros auto-import the macros they depend on (fixed earlier in [#425](https://github.com/godot-rust/godot-rust/pull/425)).

### Config / internal

- Stripped 6 unnecessary dependencies, detected by cargo-machete ([#890](https://github.com/godot-rust/godot-rust/pull/890))
- Doc CI: improved detection of unchanged code ([#877](https://github.com/godot-rust/godot-rust/pull/877))
- Tests for export APIs ([#891](https://github.com/godot-rust/godot-rust/pull/891))
- `godot_test!` macro now used consistently ([#896](https://github.com/godot-rust/godot-rust/pull/896))
- `Ord` implementation now used uniformly ([#911](https://github.com/godot-rust/godot-rust/pull/911))
- Update Android NDK (21 -> 25), workaround Rust bug ([#920](https://github.com/godot-rust/godot-rust/pull/920))
- Automate NDK detection ([#934](https://github.com/godot-rust/godot-rust/pull/934))
- Refactorings in gdnative-derive crate ([#922](https://github.com/godot-rust/godot-rust/pull/922))

## [0.10.0] - 2022-03-19

(Version `0.10.0-rc.0` has been integrated into this change set)

### Added

- Crate features
  - `serde`: support for serialization/deserialization of `VariantDispatch` and core types ([#743](https://github.com/godot-rust/godot-rust/pull/743))
  - `async`: foundation for async/await programming ([#804](https://github.com/godot-rust/godot-rust/pull/804))
  - `custom-godot`, allowing easy use of custom Godot builds
    ([#833](https://github.com/godot-rust/godot-rust/pull/833),
     [#838](https://github.com/godot-rust/godot-rust/pull/838))
- New top-level modules `init`, `log`, `profiler`, `derive`
  ([#788](https://github.com/godot-rust/godot-rust/pull/788),
   [#800](https://github.com/godot-rust/godot-rust/pull/800),
   [#811](https://github.com/godot-rust/godot-rust/pull/811))
- Geometric types
  - `Vector2` and `Vector3` constants ([#718](https://github.com/godot-rust/godot-rust/pull/718))
  - `Quat` methods ([#720](https://github.com/godot-rust/godot-rust/pull/720))
  - `Transform2D` methods ([#791](https://github.com/godot-rust/godot-rust/pull/791))
  - `Transform` methods ([#821](https://github.com/godot-rust/godot-rust/pull/821))
- Other core types
  - `VariantDispatch` struct + `Variant::dispatch()` ([#708](https://github.com/godot-rust/godot-rust/pull/708))
  - `Color` conversions: `from_hsv()`, `from_hsva()`, `to_*()` ([#729](https://github.com/godot-rust/godot-rust/pull/729))
  - `GodotString::format()` ([#816](https://github.com/godot-rust/godot-rust/pull/816))
  - `AsArg` for `Instance` + `TInstance` ([#830](https://github.com/godot-rust/godot-rust/pull/830))
  - `TRef::get_node_as()` through `NodeExt` ([#727](https://github.com/godot-rust/godot-rust/pull/727))
  - `PoolArray::to_vec()` ([#843](https://github.com/godot-rust/godot-rust/pull/843))
- Exporting
  - `#[property(get, set)]` and `Property<T>` for custom getters/setters ([#841](https://github.com/godot-rust/godot-rust/pull/841)) 
  - Array typehints ([#639](https://github.com/godot-rust/godot-rust/pull/639))
  - Type-safe registration (`Method`, `Varargs`, `FromVarargs`, ...) ([#681](https://github.com/godot-rust/godot-rust/pull/681))
  - `ClassBuilder::signal()` + `SignalBuilder` ([#828](https://github.com/godot-rust/godot-rust/pull/828))
  - `#[export]` now accepts a method name ([#734](https://github.com/godot-rust/godot-rust/pull/734))
  - `ArcData::into_inner()` ([#700](https://github.com/godot-rust/godot-rust/pull/700))
  - `MapOwned` trait + `Once<T>` user-data ([#693](https://github.com/godot-rust/godot-rust/pull/693))
  - `NoHint` for forward compatibility ([#690](https://github.com/godot-rust/godot-rust/pull/690))

### Changed

- MSRV is now 1.56 ([#833](https://github.com/godot-rust/godot-rust/pull/833), [#870](https://github.com/godot-rust/godot-rust/pull/870))
- Rust edition is now 2021 ([#870](https://github.com/godot-rust/godot-rust/pull/870))
- `euclid` vector library replaced with `glam`, no longer part of public API ([#713](https://github.com/godot-rust/godot-rust/pull/713))
- `Variant` has now a redesigned conversion API ([#819](https://github.com/godot-rust/godot-rust/pull/819))
- Type renames ([#815](https://github.com/godot-rust/godot-rust/pull/815), [#828](https://github.com/godot-rust/godot-rust/pull/828))
  - `RefInstance` -> `TInstance`
  - `RefKind` -> `Memory`
  - `ThreadAccess` -> `Ownership`
  - `TypedArray` -> `PoolArray`
  - `Element` -> `PoolElement`
  - `SignalArgument` -> `SignalParam`
- Simplified module structure
  ([#788](https://github.com/godot-rust/godot-rust/pull/788),
   [#811](https://github.com/godot-rust/godot-rust/pull/811))
  - 1 module per symbol (+prelude), no symbols at root, lower nesting depth
  - Rename `nativescript` -> `export`
  - Move `export::{Instance,RefInstance}` -> `object`
  - More details: see PR descriptions
- Geometric types API consistency ([#827](https://github.com/godot-rust/godot-rust/pull/827))
  - Rename basis vectors `x, y, z` -> `a, b, c`
  - Pass by value/ref consistency
  - `Plane` invariants ([#874](https://github.com/godot-rust/godot-rust/pull/874))
  - Other changes (see PRs)
- Method renames
  - `{String,Variant}::forget()` -> `leak()` ([#828](https://github.com/godot-rust/godot-rust/pull/828))
  - `Color::{rgb,rgba}()` -> `{from_rgb,from_rgba}()`
  - `Rid::is_valid()` -> `is_occupied()`
  - `Basis::to_scale()` -> `scale()`
  - `Basis::from_elements()` -> `from_rows()`
  - `Transform2D::from_axis_origin()` -> `from_basis_origin()`
  - `StringName::get_name()` -> `to_godot_string()` ([#874](https://github.com/godot-rust/godot-rust/pull/874))
  - `Plane::intersects_*()` -> `intersect_*()` ([#874](https://github.com/godot-rust/godot-rust/pull/874))
  - `Plane::normalize()` -> `normalized()`
  - `Plane::has_point()` -> `contains_point()` + `contains_point_eps()`
- Relax `Dictionary` key bounds: `ToVariant` -> `OwnedToVariant` ([#809](https://github.com/godot-rust/godot-rust/pull/809))
- `#[inherit]` is now optional and defaults to `Reference` ([#705](https://github.com/godot-rust/godot-rust/pull/705))
- `Instance` and `TInstance` now use `Own=Shared` by default ([#823](https://github.com/godot-rust/godot-rust/pull/823))
- Ergonomics improvements for `get_node_as()` & Co. ([#837](https://github.com/godot-rust/godot-rust/pull/837))
- Separate trait for `NativeClass` static names ([#847](https://github.com/godot-rust/godot-rust/pull/847))
- Generated docs: Godot BBCode translated to RustDoc, including intra-doc links ([#779](https://github.com/godot-rust/godot-rust/pull/779))

### Removed

(Renames listed under _Changed_, safety removals under _Fixed_)

- Crate features
  - `nativescript` ([#811](https://github.com/godot-rust/godot-rust/pull/811))
  - `bindings` ([#833](https://github.com/godot-rust/godot-rust/pull/833))
- All redundant or unnecessarily nested modules (see _Changed_)
- Deprecated symbols ([#828](https://github.com/godot-rust/godot-rust/pull/828))
  - `Reference::init_ref()` (unsound) 
  - `ClassBuilder::add_method()`, `add_method_advanced()`, `add_method_with_rpc_mode()`
  - `ScriptMethod`, `ScriptMethodFn`, `ScriptMethodAttributes`
- Never functioning or misleading
  - `FloatHint::Enum` ([#828](https://github.com/godot-rust/godot-rust/pull/828))
  - `Transform::from_axis_origin()` ([#827](https://github.com/godot-rust/godot-rust/pull/827))
- Redundant methods (cleaner API)
  - access methods for `VariantArray<Shared>` ([#795](https://github.com/godot-rust/godot-rust/pull/795))
  - `Basis::invert()`, `orthonormalize()`, `rotate()`, `tdotx()`, `tdoty()`, `tdotz()` ([#827](https://github.com/godot-rust/godot-rust/pull/827))
  - `Rid::operator_less()` ([#844](https://github.com/godot-rust/godot-rust/pull/844))
  - `StringName::operator_less()` ([#874](https://github.com/godot-rust/godot-rust/pull/874))
- Macros and attributes
  - `#[property(before_get|before_set|after_get|after_set)]`, replaced with `#[property(get|set)]` ([#874](https://github.com/godot-rust/godot-rust/pull/874))
- From `prelude`
  - macros`godot_gdnative_init`, `godot_gdnative_terminate`, `godot_nativescript_init`, `godot_site` ([#811](https://github.com/godot-rust/godot-rust/pull/811))

### Fixed

- Exports
  - Class registry for detection of already registered classes and improved errors ([#737](https://github.com/godot-rust/godot-rust/pull/737))
  - Exported properties now registered in order of declaration ([#777](https://github.com/godot-rust/godot-rust/pull/777))
  - Signal parameter types annotated in builder not propagated to Godot ([#828](https://github.com/godot-rust/godot-rust/pull/828))
- GDNative bindings
  - "Safe names" for GDNative symbols conflicting with Rust keywords
    ([#812](https://github.com/godot-rust/godot-rust/pull/812),
     [#832](https://github.com/godot-rust/godot-rust/pull/832))
  - Unresolved enum name in bindings generator ([#840](https://github.com/godot-rust/godot-rust/pull/840))
  - Silence UB warnings caused by bindgen ([#776](https://github.com/godot-rust/godot-rust/pull/776))
  - `.gdnlib` in integration tests now non-reloadable ([#746](https://github.com/godot-rust/godot-rust/pull/746))
- Core
  - Bugs in `Basis * Vector3` + `Vector3::rotated()` ([#760](https://github.com/godot-rust/godot-rust/pull/760))
  - `VariantArray`: bounds check, remove unsafe methods for VariantArray<Shared> ([#795](https://github.com/godot-rust/godot-rust/pull/795))
  - `Variant::call()` now unsafe, like `Object::call()` ([#795](https://github.com/godot-rust/godot-rust/pull/795))
  - `Rid` ([#844](https://github.com/godot-rust/godot-rust/pull/844)):
    - GDNative APIs accepting it now unsafe
    - Method `get_id()` now unsafe and null-checked
    - Fix logic error in `PartialOrd`
  - `Dictionary`:
    - Fix unsound `get()` ([#748](https://github.com/godot-rust/godot-rust/pull/748))
    - Remove `get_next()` ([#795](https://github.com/godot-rust/godot-rust/pull/795))
- Error messages conforming to Rust conventions ([#731](https://github.com/godot-rust/godot-rust/pull/731))
- Qualify identifiers in proc-macros, avoids potential naming conflicts ([#835](https://github.com/godot-rust/godot-rust/pull/835))
- Library logo ([#801](https://github.com/godot-rust/godot-rust/pull/801))

### Config

- CI overhaul: run for every PR, shorter runtime, cache ([#783](https://github.com/godot-rust/godot-rust/pull/783))
- Automatic publishing of `master` docs ([#786](https://github.com/godot-rust/godot-rust/pull/786))
- Issue templates ([#807](https://github.com/godot-rust/godot-rust/pull/807))
- Add `cargo-deny` to CI ([#849](https://github.com/godot-rust/godot-rust/pull/849))
- Add CI job which tests minimal dependencies ([#856](https://github.com/godot-rust/godot-rust/pull/856))


## [0.9.3] - 2021-02-02

### Fixed

- **The code now compiles on rustc versions not affected by https://github.com/rust-lang/rust/issues/79904**

## [0.9.2] - 2021-02-01

### Added

- Added `Instance::emplace`, a constructor that moves an existing script struct onto a new object.

- Added `Ref::by_class_name`, a method to construct Godot objects from class names.

- Added methods to recover `Ref`s or `TRef`s from instance IDs.

- Added a `Default` implementation for `NodePath`.

- Added `Add`, `AddAssign`, `Ord`, `PartialOrd`, `Index` implementations for `GodotString`.

- Added convenience methods for getting typed nodes in `gdnative-bindings`.

- Added examples for custom node plugins and Godot networking.

- Added the `#[property(no_editor)]` flag, which exports properties that can be accessed from other languages like GDScript, but aren't shown in the editor.

### Changed

- **The minimum compatible engine version is now 3.2-stable.**

- Improved readability of generated documentation for the API methods.

- Improved error messages for failed method calls.

- Proc-macros now emit compile errors rather than panic, improving the development experience.

- Documented the trade-offs of using `GodotString` vs. the `std` `String` type.

### Fixed

- `Object::callv` is now correctly marked as `unsafe`.

- Derive macro for `FromVariant` now correctly uses the actual variant name when reporting errors for enums.

- Derive macro for `OwnerToVariant` now correctly takes ownership of `self`.

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

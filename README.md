# GDNative bindings for Rust

<a href="https://godot-rust.github.io/"><img align="right" width="200" height="200" src="assets/godot-ferris.svg"></a>

[<img alt="crates.io" src="https://img.shields.io/crates/v/gdnative?logo=rust&color=A6854D" />](https://crates.io/crates/gdnative)
[<img alt="stable docs" src="https://img.shields.io/badge/docs-released-4D8AA6?&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" />](https://docs.rs/gdnative)
[<img alt="master docs" src="https://img.shields.io/badge/docs-master-4D8AA6?&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" />](https://godot-rust.github.io/docs)
[<img alt="book" src="https://img.shields.io/badge/user_guide-book-3e6ccf?&logo=read-the-docs&logoColor=white" />](https://godot-rust.github.io/book)
[<img alt="website" src="https://img.shields.io/badge/website-3e6ccf?&color=gray" />](https://godot-rust.github.io)

**godot-rust** is a Rust library that implements native bindings for the [Godot game engine](http://godotengine.org/). This allows you to develop games or other applications in Godot, while benefiting from Rust's strengths, such as its type system, scalability and performance.

> **Note**: if you are looking for a Rust binding for GDExtension (Godot 4), checkout [`gdextension`](https://github.com/godot-rust/gdextension).


## Stability

The bindings cover most of the exposed API of Godot 3.5, and are being used on a number of projects in development, but we still expect non-trivial breaking changes in the API in the coming releases. godot-rust adheres to [Cargo's semantic versioning](https://doc.rust-lang.org/cargo/reference/semver.html).

Minimum supported Rust version (MSRV) is **1.63**. We use the Rust 2021 Edition.


## Engine compatibility

Due to GDNative API not strictly following SemVer and some concepts not mapping 1:1 to Rust (default parameters),
it is difficult for a godot-rust version to remain compatible with multiple Godot versions simultaneously.

However, we support the latest stable Godot 3 minor release out-of-the-box, and allow to easily use custom engine
versions using the `custom-godot` feature flag (see [below](#custom-builds)).

Compatibility list:

* Godot 3.5.1 (works with gdnative 0.11)
* Godot 3.4 (works with gdnative 0.10, custom build for 0.11)
* Godot 3.3 (custom build)
* Godot 3.2 (custom build)

The bindings do _**not**_ support in-development Godot 4 versions.
A GDExtension binding is planned.


## Getting started

Detailed setup is explained in [the _Getting Started_ section of the book](https://godot-rust.github.io/book/getting-started.html). In case of problems, consider also reading the [FAQ](https://godot-rust.github.io/book/faq/configuration.html).

### Latest released version

This is the recommended way of using godot-rust. After `bindgen` dependencies and a current Godot version are installed, add the `gdnative` crate as a dependency, and set the crate type to `cdylib`:

```toml
[dependencies]
gdnative = "0.11"

[lib]
crate-type = ["cdylib"]
```

### Latest GitHub version

If you would like to benefit from cutting-edge features and bugfixes, you can use the GitHub version. We have a relatively sophisticated CI and test suite for basic stability, but the GitHub version is typically more experimental and less battle-tested than a `crates.io` release. We also do not guarantee any SemVer compatibility here.

```toml
[dependencies]
gdnative = { git = "https://github.com/godot-rust/godot-rust.git" }

[lib]
crate-type = ["cdylib"]
```

### Custom builds

To use the bindings with a different Godot version or a custom build of the engine, see
[Custom Godot builds](https://godot-rust.github.io/book/advanced-guides/custom-godot.html) in the user guide.

### Async/yield support

Async support is a work-in-progress, with a low-level API available in `gdnative::tasks`, if the `async` feature is enabled on `gdnative`. See [this page](https://godot-rust.github.io/book/recipes/async-tokio.html) in the book for an introduction to use the async feature with Tokio.


## Example

A typical use case is to expose your own _Native Class_, a Rust API that can be invoked from the Godot engine. The resulting native script can be attached to the scene tree, just like GDScript (`.gd` files). 

This happens via dynamic libraries and the _GDNative interface_, which will be loaded from Godot. The necessary wiring is done behind the scenes by godot-rust. A simple "Hello world" application could look like this:

```rust
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld;

#[methods]
impl HelloWorld {
    fn new(_base: &Node) -> Self {
        HelloWorld
    }

    #[method]
    fn _ready(&self, #[base] _base: &Node) {
        godot_print!("Hello, world.");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<HelloWorld>();
}

godot_init!(init);
```

### Further examples

> **Important note:**
>
> To run or edit an example, you need to build the native library for it first. Otherwise, the project will be broken. You can do so manually with `cargo build`, or use the `example.sh` shell script for convenience: `./example.sh run hello-world` or `./example.sh edit hello-world` for the editor.

The [/examples](https://github.com/godot-rust/godot-rust/tree/master/examples) directory contains several ready to use examples, complete with Godot projects and setup for easy compilation from Cargo:

- [**hello-world**](https://github.com/godot-rust/godot-rust/tree/master/examples/hello-world) - Your first project, writes to the console.
- [**spinning-cube**](https://github.com/godot-rust/godot-rust/tree/master/examples/spinning-cube) - Spin our own node in place, exposing editor properties.
- [**scene-create**](https://github.com/godot-rust/godot-rust/tree/master/examples/scene-create) - Load, instance and place scenes using Rust code.
- [**builder-export**](https://github.com/godot-rust/godot-rust/tree/master/examples/builder-export) - Export using the builder API.
- [**property-export**](https://github.com/godot-rust/godot-rust/tree/master/examples/property-export) - Export complex properties such as collections.
- [**dodge-the-creeps**](https://github.com/godot-rust/godot-rust/tree/master/examples/dodge-the-creeps) - A Rust port of the [little Godot game](https://docs.godotengine.org/en/stable/getting_started/step_by_step/your_first_game.html).
- [**signals**](https://github.com/godot-rust/godot-rust/tree/master/examples/signals) - Connect and emit signals.
- [**resource**](https://github.com/godot-rust/godot-rust/tree/master/examples/resource) - Create and use custom resources.
- [**rpc**](https://github.com/godot-rust/godot-rust/tree/master/examples/rpc) - Simple peer-to-peer networking.
- [**native-plugin**](https://github.com/godot-rust/godot-rust/tree/master/examples/native-plugin) - Create custom node plugins.

At startup, the Godot editor tries to load all resources used by the project, including the native library. If the latter isn't present, the editor will skip properties or signals associated with the missing native scripts in the scene. This causes the scene tree to be non-functional for any sample that relies on properties or signals configured in the editor.
### Third-party projects

To see a list of games and integrations developed on top of godot-rust, have a look at our list of [third-party projects](https://godot-rust.github.io/book/projects.html) in the book.


## Contributing

See the [contribution guidelines](CONTRIBUTING.md).


## License

Any contribution submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.

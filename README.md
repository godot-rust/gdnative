# GDNative bindings for Rust

<a href="https://godot-rust.github.io/"><img align="right" width="200" height="200" src="assets/godot-ferris.svg"></a>

[<img alt="crates.io" src="https://img.shields.io/crates/v/gdnative?logo=rust&color=A6854D" />](https://crates.io/crates/gdnative)
[<img alt="stable docs" src="https://img.shields.io/badge/docs-released-4D8AA6?&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" />](https://docs.rs/gdnative)
[<img alt="master docs" src="https://img.shields.io/badge/docs-master-4D8AA6?&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" />](https://godot-rust.github.io/docs)
[<img alt="book" src="https://img.shields.io/badge/user_guide-book-3e6ccf?&logo=read-the-docs&logoColor=white" />](https://godot-rust.github.io/book)
[<img alt="website" src="https://img.shields.io/badge/website-3e6ccf?&color=gray" />](https://godot-rust.github.io)

**godot-rust** is a Rust library that implements native bindings for the [Godot game engine](http://godotengine.org/). This allows you to develop games or other applications in Godot, while benefiting from Rust's strengths, such as its type system, scalability and performance.

## Stability

The bindings cover most of the exposed API of Godot 3.4, and are being used on a number of projects in development, but we still expect non-trivial breaking changes in the API in the coming releases. godot-rust adheres to [Cargo's semantic versioning](https://doc.rust-lang.org/cargo/reference/semver.html).

## Engine compatibility

We are committed to keeping compatibility with the latest stable patch releases of all minor versions of the engine, starting from Godot 3.2:
* Godot 3.4 (works out-of-the-box)
* Godot 3.3 (needs feature `custom-godot`)
* Godot 3.2 (needs feature `custom-godot`)

For versions 3.2 and 3.3, some extra steps are needed, see _Custom builds_ below.

The bindings do _**not**_ support in-development Godot 4 versions at the moment. Support is planned as the native extensions become more stable.


## Getting started

Detailed setup is explained in [the _Getting Started_ section of the book](https://godot-rust.github.io/book/getting-started.html). In case of problems, consider also reading the [FAQ](https://godot-rust.github.io/book/faq/configuration.html).

### Latest `master` version + Godot 3.4

This is the recommended way of using godot-rust, if you want to benefit from latest features.
After `bindgen` dependencies are installed, add the `gdnative` crate as a dependency, and set the crate type to `cdylib`:

```toml
[dependencies]
gdnative = { git = "https://github.com/godot-rust/godot-rust.git" }

[lib]
crate-type = ["cdylib"]
```

### Godot 3.2.3-stable

To access the last released version on crates.io, use the following. Note that there have been significant API changes since v0.9.3 -- if you are starting to use godot-rust, we recommend using the `master` version instead.

```toml
[dependencies]
gdnative = "0.9.3"

[lib]
crate-type = ["cdylib"]
```

### Custom builds

To use the bindings with a different Godot version or a custom build of the engine, see [Custom Godot builds](https://godot-rust.github.io/book/advanced-guides/custom-godot.html) in the user guide.

### Async / `yield` support

Async support is a work-in-progress, with a low-level API available in the `gdnative-async` crate. This crate is re-exported as `gdnative::tasks`, if the `async` feature is enabled on `gdnative`. See [this page](https://godot-rust.github.io/book/recipes/async-tokio.html) in the book for an introduction to use the async feature with Tokio.

## Example

A typical use case is to expose your own _Native Class_, a Rust API that can be invoked from the Godot engine. The resulting native script can be attached to the scene tree, just like GDScript (`.gd` files). 

This happens via dynamic libraries and the _GDNative interface_, which will be loaded from Godot. The necessary wiring is done behind the scenes by godot-rust. A simple "Hello world" application could look like this:

```rs
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld;

#[methods]
impl HelloWorld {
    fn new(_owner: &Node) -> Self {
        HelloWorld
    }

    #[export]
    fn _ready(&self, _owner: &Node) {
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
> Before launching the examples in the Godot editor, you must first run `cargo build` and wait for the build operations to finish successfully.
>
>At startup, the Godot editor tries to load all resources used by the project, including the native library. If the latter isn't present, the editor will skip properties or signals associated with the missing native scripts in the scene. This will cause the scene tree to be non-functional for any sample that relies on properties or signals configured in the editor.

The [/examples](https://github.com/godot-rust/godot-rust/tree/master/examples) directory contains several ready to use examples, complete with Godot projects and setup for easy compilation from Cargo:

- [**hello-world**](https://github.com/godot-rust/godot-rust/tree/master/examples/hello-world) - Your first project, writes to the console.
- [**spinning-cube**](https://github.com/godot-rust/godot-rust/tree/master/examples/spinning-cube) - Spin our own node in place, exposing editor properties.
- [**scene-create**](https://github.com/godot-rust/godot-rust/tree/master/examples/scene-create) - Load, instance and place scenes using Rust code.
- [**array-export**](https://github.com/godot-rust/godot-rust/tree/master/examples/array-export) - Export more complex properties (here arrays) from Rust.
- [**dodge-the-creeps**](https://github.com/godot-rust/godot-rust/tree/master/examples/dodge-the-creeps) - A Rust port of the [little Godot game](https://docs.godotengine.org/en/stable/getting_started/step_by_step/your_first_game.html).
- [**signals**](https://github.com/godot-rust/godot-rust/tree/master/examples/signals) - Connect and emit signals.
- [**resource**](https://github.com/godot-rust/godot-rust/tree/master/examples/resource) - Create and use custom resources.
- [**rpc**](https://github.com/godot-rust/godot-rust/tree/master/examples/rpc) - Simple peer-to-peer networking.
- [**native-plugin**](https://github.com/godot-rust/godot-rust/tree/master/examples/native-plugin) - Create custom node plugins.


### Third-party projects

To see a list of games and integrations developed on top of godot-rust, have a look at our list of [third-party projects](https://godot-rust.github.io/book/projects.html) in the book.


## Contributing

See the [contribution guidelines](CONTRIBUTING.md).


## License

Any contribution submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.

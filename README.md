# GDNative bindings for Rust

<a href="https://godot-rust.github.io/"><img align="right" width="200" height="200" src="assets/godot-ferris.svg"></a>

[![Docs Status](https://docs.rs/gdnative/badge.svg)](https://docs.rs/gdnative)

Rust bindings to the [Godot game engine](http://godotengine.org/).

**[Website](https://godot-rust.github.io/)** |
**[User Guide](https://godot-rust.github.io/book/)** | **[API Documentation](https://docs.rs/gdnative/0.9.1/gdnative/)**

## Stability

The bindings cover most of the exposed API of Godot 3.2, and are being used on a number of projects in development, but we still expect non-trivial breaking changes in the API in the coming releases.

## Engine compatibility

We are serious about engine compatibility. We are committed to keeping compatibility with the latest stable patch releases of all minor versions of the engine, starting from Godot 3.2.

The current minimum compatible version, with `api.json` replacement, is Godot 3.2-stable. Changes to this will be considered a breaking change, and will be called out in the release notes.

The bindings do *not* support Godot 4.0 (`master` branch) currently.

## Requirements

The generator makes use of `bindgen`, which depends on Clang. Instructions for installing `bindgen`'s dependencies for popular OSes can be found in their documentation: https://rust-lang.github.io/rust-bindgen/requirements.html.

`bindgen` may complain about a missing `llvm-config` binary, but it is not actually required to build the `gdnative` crate. If you see a warning about `llvm-config` and a failed build, it's likely that you're having a different problem!

## Usage

### Godot 3.2.3-stable

After `bindgen` dependencies are installed, add the `gdnative` crate as a dependency, and set the crate type to `cdylib`:

```toml
[dependencies]
gdnative = "0.9.1"

[lib]
crate-type = ["cdylib"]
```

### Other versions or custom builds

The bindings are currently generated from the API description of Godot 3.2.3-stable by default. To use the bindings with another version or a custom build, see [Using custom builds of Godot](https://godot-rust.github.io/book/advanced-guides/custom-bindings.html) in the user guide.

## Example

The most general use-case of the bindings will be to interact with Godot using the generated wrapper
classes, as well as providing custom functionality by exposing Rust types as *NativeScript*s.

NativeScript is an extension for GDNative that allows a dynamic library to register "script classes"
to Godot.

As is tradition, a simple "Hello World" should serve as an introduction. For a full tutorial, check out ["Getting Started" from the user guide](https://godot-rust.github.io/book/getting-started.html)!

```rust
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
        godot_print!("hello, world.");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<HelloWorld>();
}

godot_init!(init);
```

### Further examples

The [/examples](https://github.com/godot-rust/godot-rust/tree/master/examples) directory contains several ready to use examples, complete with Godot projects and setup for easy compilation from Cargo:

- [/examples/hello_world](https://github.com/godot-rust/godot-rust/tree/master/examples/hello_world) - Your first project, writes to the console
- [/examples/spinning_cube/](https://github.com/godot-rust/godot-rust/tree/master/examples/spinning_cube) - Spinning our own node in place, exposing editor properties.
- [/examples/scene_create](https://github.com/godot-rust/godot-rust/tree/master/examples/scene_create) - Shows you how to load, instance and place scenes using Rust code
- [/examples/signals](https://github.com/godot-rust/godot-rust/tree/master/examples/signals) - Shows you how to handle signals.
- [/examples/resource](https://github.com/godot-rust/godot-rust/tree/master/examples/resource) - Shows you how to create and use custom resources.

## Third-party resources

### Tutorials

- Step by step guide - [Up and running with Rust and Godot: A basic setup](https://hagsteel.com/posts/godot-rust/)

### Open-source projects

- Pong - https://github.com/you-win/godot-pong-rust
- Air Combat - https://github.com/paytonrules/AirCombat - This [Godot Tutorial](https://devga.me/tutorials/godot2d/) ported to Rust.
- Action RPG - https://github.com/MacKarp/Rust_Action_RPG_Tutorial - This [Godot Tutorial](https://www.youtube.com/playlist?list=PL9FzW-m48fn2SlrW0KoLT4n5egNdX-W9a) ported to Rust.

## Contributing

See the [contribution guidelines](CONTRIBUTING.md)

## License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.

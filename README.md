# GDNative bindings for Rust

<a href="https://godot-rust.github.io/"><img align="right" width="200" height="200" src="assets/godot-ferris.svg"></a>

[![Docs Status](https://docs.rs/gdnative/badge.svg)](https://docs.rs/gdnative)


**[Website](https://godot-rust.github.io)** |
**[User Guide](https://godot-rust.github.io/book)** | **[Stable Docs](https://docs.rs/gdnative)** | **[Latest Docs](https://godot-rust.github.io/docs)**

**godot-rust** is a Rust library that implements native bindings for the [Godot game engine](http://godotengine.org/). This allows you to develop games or other applications in Godot, while benefiting from Rust's strengths, such as its type system, scalability and performance.

## Stability

The bindings cover most of the exposed API of Godot 3.2, and are being used on a number of projects in development, but we still expect non-trivial breaking changes in the API in the coming releases. godot-rust adheres to [Cargo's semantic versioning](https://doc.rust-lang.org/cargo/reference/semver.html).

## Engine compatibility

We are committed to keeping compatibility with the latest stable patch releases of all minor versions of the engine, starting from Godot 3.2.

The current minimum compatible version, with `api.json` replacement, is Godot 3.2. Godot 3.3 is supported as well. Changes to this will be considered a breaking change, and will be called out in the release notes.

The bindings do _**not**_ support Godot 4.0 currently. Support is planned as the native extensions become more stable.

## Requirements

The generator makes use of `bindgen`, which depends on Clang. Instructions for installing `bindgen`'s dependencies for popular OSes can be found in their documentation: https://rust-lang.github.io/rust-bindgen/requirements.html.

`bindgen` may complain about a missing `llvm-config` binary, but it is not actually required to build the `gdnative` crate. If you see a warning about `llvm-config` and a failed build, it's likely that you're having a different problem!

### 'Header not found' errors

When building the library, `bindgen` may produce errors that look like this:

```
godot-rust\gdnative-sys/godot_headers\gdnative/string.h:39:10: fatal error: 'wchar.h' file not found
```

This means that `bindgen` was unable to find the C system headers for your platform. If you can locate the headers manually, you may try setting the `C_INCLUDE_PATH` environment variable so `libclang` could find them. If on Windows, you may try building from the Visual Studio "developer console", which should setup the appropriate variables for you.

## Usage

### Godot 3.2.3-stable

After `bindgen` dependencies are installed, add the `gdnative` crate as a dependency, and set the crate type to `cdylib`:

```toml
[dependencies]
gdnative = "0.9.3"

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
        godot_print!("Hello, world.");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<HelloWorld>();
}

godot_init!(init);
```

### Further examples


> ### **IMPORTANT NOTE**
>
> Before launching the examples in the godot editor, you must first run `cargo build` and wait for the build operations to finish successfully.
>
>At startup, the Godot editor tries to load all resources used by the project, including the native binary. If it isn't present, the editor skips properties or signals associated with the missing NativeScripts in the scene. This will cause the scene tree to be non-functional for any sample that relies on properties or signals configured in the editor.

The [/examples](https://github.com/godot-rust/godot-rust/tree/master/examples) directory contains several ready to use examples, complete with Godot projects and setup for easy compilation from Cargo:

- [/examples/hello_world](https://github.com/godot-rust/godot-rust/tree/master/examples/hello_world) - Your first project, writes to the console
- [/examples/spinning_cube/](https://github.com/godot-rust/godot-rust/tree/master/examples/spinning_cube) - Spinning our own node in place, exposing editor properties.
- [/examples/scene_create](https://github.com/godot-rust/godot-rust/tree/master/examples/scene_create) - Shows you how to load, instance and place scenes using Rust code
- [/examples/signals](https://github.com/godot-rust/godot-rust/tree/master/examples/signals) - Shows you how to handle signals.
- [/examples/resource](https://github.com/godot-rust/godot-rust/tree/master/examples/resource) - Shows you how to create and use custom resources.
- [/examples/native_plugin](https://github.com/godot-rust/godot-rust/tree/master/examples/native_plugin) - Shows you how to create custom node plugins.

## Third-party resources

### Tools and integrations

- [godot-egui](https://github.com/setzer22/godot-egui) (setzer22, jacobsky) - combine the [egui](https://github.com/emilk/egui) library with Godot 
- [ftw](https://github.com/macalimlim/ftw) (macalimlim) - manage your godot-rust projects from the command line

### Open-source games

- [Action RPG](https://github.com/MacKarp/Rust_Action_RPG_Tutorial) (MacKarp) - this [Godot tutorial](https://www.youtube.com/playlist?list=PL9FzW-m48fn2SlrW0KoLT4n5egNdX-W9a) ported to Rust.
- [Air Combat](https://github.com/paytonrules/AirCombat) (paytonrules) - this [Godot tutorial](https://devga.me/tutorials/godot2d/) ported to Rust.
- [Simple single-player Blackjack](https://github.com/paytonrules/Blackjack) (paytonrules)
- [Pong](https://github.com/you-win/godot-pong-rust) (you-win)

### Tutorials

- Step by step guide - [Up and running with Rust and Godot: A basic setup](https://hagsteel.com/posts/godot-rust/)


## Contributing

See the [contribution guidelines](CONTRIBUTING.md).

## License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.

# GDNative bindings for Rust

<img align="right" width="200" height="200" src="assets/godot-ferris.svg">

[![Docs Status](https://docs.rs/gdnative/badge.svg)](https://docs.rs/gdnative)

Rust bindings to the [Godot game engine](http://godotengine.org/).

**[API Documentation](https://docs.rs/gdnative/0.8.1/gdnative/)**

- Note that this generally matches the [Godot API](https://docs.godotengine.org/en/3.2/classes/) but you have to do casting between classes and subclasses manually.

## Stability

The bindings cover most of the exposed API of Godot 3.2, and are being used on a number of projects in development, but we still expect non-trivial breaking changes in the API in the coming releases.

## Engine compatibility

We are serious about engine compatibility. We are committed to keeping compatibility with the latest stable patch releases of all minor versions of the engine, starting from Godot 3.2.

The current minimum compatible version, with `api.json` replacement, is Godot 3.1.1-stable. Changes to this will be considered a breaking change, and will be called out in the release notes.

## Requirements

The generator makes use of `bindgen`, which depends on Clang. Instructions for installing `bindgen`'s dependencies for popular OSes can be found in their documentation: https://rust-lang.github.io/rust-bindgen/requirements.html.

## Usage

### Godot 3.2

After `bindgen` dependencies are installed, add the `gdnative` crate as a dependency, and set the crate type to `cdylib`:

```toml
[dependencies]
gdnative = "0.8"

[lib]
crate-type = ["cdylib"]
```

### Other versions or custom builds

The bindings are currently generated from the API description of Godot 3.2 by default. To use the bindings with another version or a custom build, one currently has to use the bindings as a local dependency:

```
# Clone the repository and check out version 0.8.1
git clone https://github.com/godot-rust/godot-rust/
cd godot-rust
git checkout 0.8.1

# Update the API description file
godot --gdnative-generate-json-api bindings_generator/api.json
```

Then, add the `gdnative` crate as a local dependency instead:

```toml
[dependencies]
gdnative = { path = "path/to/godot-rust/gdnative" }
```

## Example

The most general use-case of the bindings will be to interact with Godot using the generated wrapper
classes, as well as providing custom functionality by exposing Rust types as *NativeScript*s.

NativeScript is an extension for GDNative that allows a dynamic library to register "script classes"
to Godot.

(The following section is a very quick-and-dirty rundown of how to get started with the Rust bindings.
For a more complete and detailed introduction see the [Godot documentation page](https://docs.godotengine.org/en/latest/tutorials/plugins/gdnative/gdnative-c-example.html).)

As is tradition, a simple "Hello World" should serve as an introduction. A copy of this "hello world" project can be found in the [`examples`](examples/hello_world) folder.

### The project setup

Starting with an empty Godot 3.2 project, a `cargo` project can be created inside the project folder.

```sh
cargo init --lib
```

To use the GDNative bindings in your project you have to add the `gdnative` crate as a dependency.

```toml
[dependencies]
gdnative = "0.8"
```

Since GDNative can only use C-compatible dynamic libraries, the crate type has to be set accordingly.

```toml
[lib]
crate-type = ["cdylib"]
```

### The Rust source code

In the `src/lib.rs` file should have the following contents:

```rust
use gdnative::prelude::*;

/// The HelloWorld "class"
#[derive(NativeClass)]
#[inherit(Node)]
pub struct HelloWorld;

// __One__ `impl` block can have the `#[methods]` attribute, which will generate
// code to automatically bind any exported methods to Godot.
#[methods]
impl HelloWorld {

    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        HelloWorld
    }

    // To make a method known to Godot, use the #[export] attribute.
    // In Godot, script "classes" do not actually inherit the parent class.
    // Instead, they are "attached" to the parent object, called the "owner".
    //
    // In order to enable access to the owner, it is passed as the second
    // argument to every single exposed method. As a result, all exposed
    // methods MUST have `owner: BaseClass` as their second arguments,
    // before all other arguments in the signature.
    #[export]
    fn _ready(&self, _owner: &Node) {
        // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.
        godot_print!("hello, world.");
    }
}

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    handle.add_class::<HelloWorld>();
}

// macro that creates the entry-points of the dynamic library.
godot_init!(init);
```

### Creating the NativeScript instance.

After building the library with `cargo build`, the resulting library should be in the `target/debug/` folder.

All NativeScript classes live in a GDNative library.
To specify the GDNative library, a `GDNativeLibrary` resource has to be created.
This can be done in the "Inspector" panel in the Godot editor by clicking the "new resource" button in the top left.

With the `GDNativeLibrary` resource created, the path to the generated binary can be set.

**NOTE**: Resources do not autosave, so after specifying the path, make sure to save
the `GDNativeLibrary` resource by clicking the "tool" button in the Inspector panel in the top right.

Now the `HelloWorld` class can be added to any node by clicking the "add script" button.
In the popup-select the "NativeScript" option and set the class name to "HelloWorld".

**NOTE**: After creation, the NativeScript resource does not automatically point to the `GDNativeLibrary` resource.
Make sure to set click the "library" field in the Inspector and "load" the library.

### Further examples

The [/examples](https://github.com/godot-rust/godot-rust/tree/master/examples) directory contains several ready to use examples, complete with Godot projects and setup for easy compilation from Cargo:

- [/examples/hello_world](https://github.com/godot-rust/godot-rust/tree/master/examples/hello_world) - Your first project, writes to the console
- [/examples/spinning_cube/](https://github.com/godot-rust/godot-rust/tree/master/examples/spinning_cube) - Spinning our own node in place, exposing editor properties.
- [/examples/scene_create](https://github.com/godot-rust/godot-rust/tree/master/examples/scene_create) - Shows you how to load, instance and place scenes using Rust code
- [/examples/signals](https://github.com/godot-rust/godot-rust/tree/master/examples/signals) - Shows you how to handle signals.

## Third-party resources

Several third-party resources have been created for the bindings. Open a PR to have yours included here!

### Tutorials

- In depth Hello World tutorial - [Gorgeous Godot games in Rust](https://medium.com/@recallsingularity/gorgeous-godot-games-in-rust-1867c56045e6?source=friends_link&sk=c2fd85689b4638eae4d91b743439c75f)
- Step by step guide - [Up and running with Rust and Godot: A basic setup](https://hagsteel.com/posts/godot-rust/)
- Writup/Tutorial on how to port GDScript to Rust - [Porting Godot Games to Rust](https://paytonrules.com/post/games-in-rust-with-godot-part-one/)
- Guide and sample CI powered multi-platform Rust/GDNative based boilerplate project - https://github.com/tommywalkie/sample-godot-rust-app

### Open-source projects

- Pong - https://github.com/you-win/godot-pong-rust
- Air Combat - https://github.com/paytonrules/AirCombat - The end result of the porting tutorial by @paytonrules.

### Utilities

- Auto Setup Script (as git dependency) - https://gitlab.com/ardawan-opensource/gdnative-rust-setup

## Contributing

See the [contribution guidelines](CONTRIBUTING.md)

## License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be licensed under the [MIT license](LICENSE.md), without any additional terms or conditions.

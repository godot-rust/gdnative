# godot-headers

This repository contains C headers for
[**Godot Engine**](https://github.com/godotengine/godot)'s *GDNative* API,
which can be used to write *NativeScripts*.

> `GDNative` enables the use of dynamically linked libraries inside of
> [**Godot**](https://github.com/godotengine/godot).

> `NativeScript` uses GDNative to implement scripts backed by native code.

- [**Versioning**](#versioning)
- [**Getting Started**](#getting-started)
- [**FAQ**](#faq)
- [**Updating Headers**](#updating-headers)

## Versioning

This repositories follows the same branch versioning as the main [Godot Engine
repository](https://github.com/godotengine/godot):

- `master` tracks the current development branch. As this is a moving target,
  the headers in this repository may not always be fully in sync with upstream.
  See [**Updating Headers**](#updating-headers) if you need to bring
  them up to date.
- `3.x` tracks the development of the next 3.x minor release. Like `master`, it
  might not always be fully up-to-date with upstream.
- Other versioned branches (e.g. `3.3`, `3.2`) track the latest stable release
  in the corresponding branch.

Stable releases are also tagged on this repository:
[**Tags**](https://github.com/godotengine/godot-headers/tags).

**For any project built against a stable release of Godot, we recommend using
this repository as a Git submodule, checking out the specific tag matching your
Godot version.**

## Getting Started

| **Build latest version of Godot** | [**GitHub**](https://github.com/godotengine/godot) | [**Docs**](https://docs.godotengine.org/en/latest/development/compiling/index.html) |
| --- | --- | --- |

### Clone `godot-headers` into Library

Clone `godot-headers` under `SimpleLibrary/`

```bash
cd SimpleLibrary
git clone https://github.com/godotengine/godot-headers
```

Note that the master branch of this repository contains the headers for the
latest Godot `master` branch. See [**Versioning**](#versioning) for details.
You can use `-b <version>` to the above Git clone command to retrieve a specific
branch or tag (e.g. `-b 3.x` or `-b godot-3.3.3-stable`).

```bash
[SimpleLibrary]
  ├── lib/
  └── src/
```

### Create Script

Create `test.c` under `SimpleLibrary/src/`.

<details>

```c
#include <gdnative/gdnative.h>
#include <nativescript/godot_nativescript.h>

#include <stdio.h>

void *test_constructor(godot_object *obj, void *method_data) {
	printf("test.constructor()\n");
	return 0;
}

void test_destructor(godot_object *obj, void *method_data, void *user_data) {
	printf("test.destructor()\n");
}

/** func _ready() **/
godot_variant test_ready(godot_object *obj, void *method_data, void *user_data, int num_args, godot_variant **args) {
	godot_variant ret;
	godot_variant_new_nil(&ret);

	printf("_ready()\n");

	return ret;
}

/** Library entry point **/
void GDN_EXPORT godot_gdnative_init(godot_gdnative_init_options *o) {
}

/** Library de-initialization **/
void GDN_EXPORT godot_gdnative_terminate(godot_gdnative_terminate_options *o) {
}

/** Script entry (Registering all the classes and stuff) **/
void GDN_EXPORT godot_nativescript_init(void *desc) {
	printf("nativescript init\n");

	godot_instance_create_func create_func = {
		.create_func = &test_constructor,
		.method_data = 0,
		.free_func   = 0
	};

	godot_instance_destroy_func destroy_func = {
		.destroy_func = &test_destructor,
		.method_data  = 0,
		.free_func    = 0
	};

	godot_nativescript_register_class(desc, "SimpleClass", "Node", create_func, destroy_func);

	{
		godot_instance_method method = {
			.method = &test_ready,
			.method_data = 0,
			.free_func = 0
		};

		godot_method_attributes attr = {
			.rpc_type = GODOT_METHOD_RPC_MODE_DISABLED
		};

		godot_nativescript_register_method(desc, "SimpleClass", "_ready", attr, method);
	}
}

godot_variant GDN_EXPORT some_test_procedure(void *data, godot_array *args) {
	godot_variant ret;
	godot_variant_new_int(&ret, 42);

	godot_string s;
	godot_string_new_with_wide_string(&s, L"Hello World", 11);
	godot_print(&s);

	godot_string_destroy(&s);

	return ret;
}
```

</details>

Expand *Details* for example code.

### Compile Library

On Linux:

```bash
clang -g -fPIC -c src/test.c -I/path/to/godot/headers/ -o src/test.os
clang -g -shared src/test.os -o lib/test.so
```

On MacOS:

```bash
clang -g -fPIC -c src/test.c -I/path/to/godot/headers/ -o src/test.os
clang -g -shared -framework Cocoa -Wl,-undefined,dynamic_lookup src/test.os -o lib/test.dylib
```

- `-g` is for debugging information.
- Use `godot_nativescript_*` methods only in the `nativescript_init()` function.

### Create GDNativeLibrary Resource
The GDNativeLibrary resource contains links to the libraries for each platform.

1. Create a new resource in memory and edit it.
2. Select `Resource > GDNativeLibrary`.
3. Set the library file for your platform inside the inspector.
4. Save the edited resource as a `.tres`

<details>

![](images/faq/dllibrary_create_new_resource.png?raw=true)

![](images/faq/dllibrary_create_new_dllibrary.png?raw=true)

![](images/faq/dllibrary_save_as_resource.png?raw=true)

*Note*: Remember to save `GDNativeLibrary` as `.gdnlib`

</details>

Expand *Details* for screenshots.

### Using GDNativeLibrary in GDScript

```gdscript
extends Node

func _ready():
	var gdn = GDNative.new()
	gdn.library = load("res://lib/libtest.tres")

	gdn.initialize()

	var res = gdn.call_native("standard_varcall", "some_test_procedure", [])

	print("result: ", res)

	gdn.terminate()
```

### Attaching GDNativeLibrary to a Node

1. Attach a new script to a node.
2. In the pop-up dialog, choose NativeScript in the `Language` menu.
3. Enable built-in script, or create a `.gdn` file, which only contains a name.
4. Specify the `Class Name`.
5. Press `Create`.

The GDNativeLibrary field in a NativeScript is empty by default.


<details>

![](images/faq/create_dlscript.png?raw=true)

![](images/faq/set_script_dllibrary.png?raw=true)

</details>

Expand *Details* for screenshots.

## FAQ

**What is the difference between `GDNative` and `NativeScript`?**

`GDNative` is a new class that can call native functions in libraries.
GDScript / VisualScript / C#, etc, are able to use this class.

Godot treats `NativeScript` as a scripting language, enabling the
use of GDNative to implement scripts backed by native code.

**Which languages are binding as a NativeScript?**

[**C++**](https://github.com/godotengine/godot-cpp),
[**D**](https://github.com/godot-d/godot-d),
[**Nim**](https://github.com/pragmagic/godot-nim)

**Can you debug NativeScripts?**

You must compile the library with debug
symbols, and then you can use your debugger as usual.

**Can you use one GDNativeLibrary for all NativeScripts?**

You can! ✨

**What is the reason behind the name "GDNative"?**

GDNative was originally named "cscript" because it exposes a C API, but people
mistook a relation to C#, which is sometimes abbreviated as "cs". Then named
"DLScript", but that brought up some confusion, so we settled with GDNative. 📖

## Updating Headers

See [**Versioning**](#versioning) for details on the Godot versions tracked by
each branch of this repository.

If the relevant branch is not up-to-date for your needs, or if you want to sync
the headers with your own modified version of Godot, here is the update
procedure used to sync this repository with upstream releases:

- Compile [Godot Engine](https://github.com/godotengine/godot) at the specific
  version/commit which you are using.
- Use the compiled executable to generate the `api.json` file with:
  `godot --gdnative-generate-json-api api.json`
- Copy the file `modules/gdnative/gdnative_api.json` to this repository.
- Copy the files and folders from `modules/gdnative/include` to this repository,
  overwriting existing content. (To be sure to be in sync, you can delete the
  folders of this repository first, then copy the upstream folders in place.)
  Make sure that you compiled the correct Godot version so that the generated
  `gdnative_api_struct.gen.h` is up-to-date.

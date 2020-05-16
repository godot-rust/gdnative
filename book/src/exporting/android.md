# Android

**Disclaimer**: _Currently, the following steps are tested and confirmed to work on Linux only._

In order to export to Android, we need to compile our Rust source for the appropriate targets. Unlike compiling for our native targets, there are a few extra steps involved with cross-compiling for another target.

## Installing prerequisites

First, we need to install the **Android SDK** with **NDK** enabled. This contains the necessary tools for each architecture. Once the Android SDK is installed, open Editor Settings in the Godot GUI (*Editor > Editor Settings > Export > Android*) and set the **absolute paths** to `adb`, `jarsigner`, and the debug keystore (`debug.keystore`), all of which can be found in the Android SDK installation.

Then, we'll install the Rust toolchains for the targets we want to support:

```bash
rustup target add aarch64-linux-android    # for arm64 (64-bit)
rustup target add x86_64-linux-android     # for x86_64 (64-bit)
```

### 32-bit targets

The `aarch64-linux-android` and `x86_64-linux-android` toolchains are our top priorities, because Google [has been requiring 64-bit binaries](https://android-developers.googleblog.com/2019/01/get-your-apps-ready-for-64-bit.html) for all new apps on Play Store since August 2019, and will stop serving 32-bit apps in 2021. If you, nevertheless, want to support 32-bit targets, there are a few more dependencies to install.

> ### A bit of context
>
> There are two major CPU providers in the Android ecosystem: ARM and Intel.
>
> They were primarily supporting 32-bit OS, with notably [**ARMv7**](https://en.wikipedia.org/wiki/ARM_architecture#32-bit_architecture) and [**x86**](https://en.wikipedia.org/wiki/X86) architectures, until they started supporting 64-bit OS, by introducing [**ARMv8-A**](https://en.wikipedia.org/wiki/ARM_architecture#64/32-bit_architecture) (often called **ARM64**) and [**x86-64**](https://en.wikipedia.org/wiki/X86-64) (often called **Intel 64** or **AMD64**, in reference to a [long-time conflict](https://en.wikipedia.org/wiki/X86-64#History_2) between Intel and AMD).
>
> **Aarch64** is the 64-bit execution state that is introduced in ARM64 chips. [**i686**](https://en.wikipedia.org/wiki/P6_%28microarchitecture%29) (also called **P6**) is actually the sixth-generation Intel x86 microarchitecture.
>
> Generally speaking, 32-bit programs can run on 64-bit systems, but 64-bit programs won't run on 32-bit systems.

#### Rust toolchains for 32-bit targets

```bash
rustup target add armv7-linux-androideabi  # for armv7 (32-bit)
rustup target add i686-linux-android       # for x86 (32-bit)
```

#### `gcc` libraries for cross-compilation

On Windows, we will need to setup a [32-bit/64-bit compatible MinGW](https://sourceforge.net/projects/mingw-w64/) instance.

On UNIX-like systems, the required packages are usually available under different names in the package managers for each distribution. On Debian-based Linuxes (including Ubuntu), for example, the required libraries can be installed using `apt`:

```bash
apt-get update
apt-get install g++-multilib gcc-multilib libc6-dev-i386 -y
```

## Setting up Cargo

To make Cargo aware of the proper platform-specific linkers that it needs to use for Android targets, we need to put the paths to the binaries in the Cargo configuration file, which can be found (or created) at `$HOME/.cargo/config` on UNIX-like systems, or `%USERPROFILE%\.cargo\config` on Windows), using [`[target]` tables](https://doc.rust-lang.org/cargo/reference/config.html#target):

```toml
[target.armv7-linux-androideabi]
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi29-clang"
```

... where the value of `linker` is an **absolute path** to the Android SDK linker for the target triple. Assuming `$ANDROID_SDK_ROOT` is the Android SDK path, these binaries can be found at:

- Windows: `$ANDROID_SDK_ROOT\ndk\<NDK-VERSION>\toolchains\llvm\prebuilt\windows-x86_64\bin\`, where `<NDK-VERSION>` is the installed NDK instance version
- UNIX-like systems: `$ANDROID_SDK_ROOT/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/`

Repeat for all targets installed in the previous step, until we get something that looks like:

```toml
# Example configuration on an UNIX-like system. `29` is the Android API version.

[target.armv7-linux-androideabi]
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi29-clang"

[target.aarch64-linux-android]
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"

[target.i686-linux-android]
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android29-clang"

[target.x86_64-linux-android]
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android29-clang"
```

## Setting up environment variables for `gdnative-sys`

The `gdnative-sys` crate can infer include paths for Android targets, but it requires the following environment variables:

- `$JAVA_HOME`, which should point to the installed JDK instance.
- `$ANDROID_SDK_ROOT`, which should point to the Android SDK root (which contains the `ndk-bundle` directory).

Depending on your installation, these environment variables might have already been set. Otherwise, the variables may be set in bash:

```bash
export JAVA_HOME=/path/to/jdk
export ANDROID_SDK_ROOT=/path/to/android/sdk
```

... or in PowerShell on Windows:

```powershell
$env:JAVA_HOME = "C:\path\to\jdk"
$env:ANDROID_SDK_ROOT = "C:\path\to\android\sdk"
```

## Building the GDNative library

Finally, we can now build the GDNative library with Cargo for one or multiple targets:

```bash
cargo build --release --target x86_64-linux-android
```

**Important note**: ARM and x86 are, by design, different architectures. It is normal to get errors while running `cargo test` with a Rust library targeting ARMv7 on a x86-64 CPU, for example, since the CPU is unable to handle it.

## Exporting in Godot

### Linking to Android binaries in `.gdns`

After building the GDNative libraries, we need to link them to Godot, by adding new entries in the GDNative library declaration file (`*.gdnlib`) for `Android.armeabi-v7a` (ARMv7),  `arm64-v8a` (ARM64), `Android.x86` (x86) and/or `Android.x86_64` (x86-64), depending of the toolchains we actually used in previous steps:

```
[entry]

Android.armeabi-v7a="res://target/armv7-linux-androideabi/release/lib.so"
Android.arm64-v8a="res://target/aarch64-linux-android/release/lib.so"
Android.x86="res://target/i686-linux-android/release/lib.so"
Android.x86_64="res://target/x86_64-linux-android/release/lib.so"

[dependencies]

Android.armeabi-v7a=[  ]
Android.arm64-v8a=[  ]
Android.x86=[  ]
Android.x86_64=[  ]
```

### APK signing for publication

Usually, we can choose between releasing an app in **Debug** or **Release** mode. However, the Release mode is required when officially releasing to Play Store.

In order to configure Godot to sign Release APKs, we'll first need to generate a project-specific Release keystore using `keytool`, and set up an alias and a single password (as explained in the [Godot docs](https://docs.godotengine.org/en/3.2/getting_started/workflow/export/exporting_for_android.html#exporting-for-google-play-store), `-storepass` and `-keypass` option values must be the same):

```bash
keytool -genkeypair -v -keystore path/to/my.keystore -alias some-alias -keyalg RSA -keysize 2048 -validity 10000 -storepass my-password -keypass my-password
```

Then, we will register its path in Export Settings (*Project > Export*) or `export_presets.cfg`. Please note that passwords entered in the GUI will be stored in `export_presets.cfg`. Be sure to not commit it into any VCS!

```
# Remember to not commit the password as is in VCS!
keystore/release="path/to/my.keystore"
keystore/release_user="some-alias"
keystore/release_password="my-password"
```

### Exporting

Finally, we can now export the project using the GUI (*Project > Export... > Android (Runnable)*) and uncheck "*Export with Debug*" in GUI when being asked to enter APK file name. We may also use one of the following commands from the CLI to do the same:

```bash
# Debug mode
godot --export-debug "Android" path/to/my.apk

# Release mode
godot --export "Android" path/to/my.apk
```

When trying to install the app directly from the APK on an Android device, Play Protect may display a warning explaining that _the app developers are not recognized, so the app may be unsafe_. This is the expected behavior for an APK in Release mode that isn't actually released on Play Store.

If not planning to release on Play Store, one may file an appeal from Play Protect using [a form provided by Google](https://support.google.com/googleplay/android-developer/contact/protectappeals).
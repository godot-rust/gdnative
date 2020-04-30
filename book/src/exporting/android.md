# Android

To export GDNative/Rust based games on Android, we first need to compile Rust source for the appropriate targets. Since August 1, 2019, Google [now requires to have a 64-bit compatible version](https://android-developers.googleblog.com/2019/01/get-your-apps-ready-for-64-bit.html) for any new published apps on Play Store, and will stop serving 32-bit apps in 2021. Before diving into setting up Cargo and Godot Engine, we need to understand what are the existing Android CPU architectures so the following sections will be easier to follow.

## A bit of context

Historically, there are two major CPU providers in Android ecosystem : ARM and Intel. 

They were primarily supporting 32-bit OS, with notably [**ARMv7**](https://en.wikipedia.org/wiki/ARM_architecture#32-bit_architecture) and [**x86**](https://en.wikipedia.org/wiki/X86) architectures, until they started supporting 64-bit OS, by introducing [**ARMv8-A**](https://en.wikipedia.org/wiki/ARM_architecture#64/32-bit_architecture) (often called **ARM64**) and [**x86-64**](https://en.wikipedia.org/wiki/X86-64) (often called **Intel 64** or **AMD64**, in reference to a [long-time conflict](https://en.wikipedia.org/wiki/X86-64#History_2) between Intel and AMD). 

**Aarch64** is the 64-bit execution state that is introduced in ARM64 chips. [**i686**](https://en.wikipedia.org/wiki/P6_%28microarchitecture%29) (also called **P6**) is actually the sixth-generation Intel x86 microarchitecture.

Generally speaking, 32-bit programs can run on 64-bit systems, but 64-bit programs won't run on 32-bit systems.

## Setting up Cargo

**Disclaimer** : _Currently, the following steps are tested and confirmed to work on Linux only._

First, we need to install **Android SDK** with **NDK** enabled, which comes with adequate **LLVM** toolchains with archivers (`ar`) and linkers (`linker`) for each architecture.

Install the Rust toolchains for the targets we want to support. Given the previously mentioned statement from Google, `aarch64-linux-android` and `x86_64-linux-android` toolchains are our top priority.

```bash
rustup target add armv7-linux-androideabi  # for armv7 (32-bit)
rustup target add aarch64-linux-android    # for arm64 (64-bit)
rustup target add i686-linux-android       # for x86 (32-bit)
rustup target add x86_64-linux-android     # for x86_64 (64-bit)
```

Assuming **$ANDROID_HOME** is the Android SDK path, edit (or create, if not exists) a Cargo configuration file (`$HOME/.cargo/config` on UNIX or `%USERPROFILE%\.cargo\config` on Windows) and set the proper archivers and linkers **absolute paths** for the right Rust toolchains. These can be found on :

- Windows : `$ANDROID_HOME\ndk\<NDK-VERSION>\toolchains\llvm\prebuilt\windows-x86_64\bin\` (assuming `<NDK-VERSION>` is the installed NDK instance version)
- UNIX : `$ANDROID_HOME/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/`

Here is an UNIX based example (where `29` is the Android API version) :

```toml
[target.armv7-linux-androideabi]
ar = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/arm-linux-androideabi-ar"
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi29-clang"

[target.aarch64-linux-android]
ar = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar"
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android29-clang"

[target.i686-linux-android]
ar = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android-ar"
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/i686-linux-android29-clang"

[target.x86_64-linux-android]
ar = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android-ar"
linker = "/usr/local/lib/android/sdk/ndk-bundle/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android29-clang"
```

In order to make `gcc` properly cross-compile C and C++ libraries, we will need to setup a [32-bit/64-bit compatible MinGW](https://sourceforge.net/projects/mingw-w64/) instance, if using Windows, or install a few packages if using Linux :

```bash
apt-get update
apt-get install g++-multilib gcc-multilib libc6-dev-i386 -y
```

Compiling `gdnative-sys` crate for Android targets will also need some bindings with the **Java Native Interface (JNI)**, which is defined in some C++ headers :

- `jni.h` 
  - Windows : `$JAVA_HOME\include\` (assuming `$JAVA_HOME` is the installed JDK instance)
  - UNIX : `$JAVA_HOME/include/`
- `jni_md.h`
  - Windows : `$JAVA_HOME\include\win32\` (assuming `$JAVA_HOME` is the installed JDK instance)
  - UNIX : `$JAVA_HOME/include/linux/`

We can use `C_INCLUDE_PATH` as a environment variable, with both the `jni.h` and `jni_md.h` parent folder paths as values.

```bash
# Bash
C_INCLUDE_PATH=.:$JAVA_HOME/include/:$JAVA_HOME/include/linux/

# Powershell
$env:C_INCLUDE_PATH = "$env:JAVA_HOME\include;$env:JAVA_HOME\include\win32"
```

Finally, build the GDNative library with Cargo for one or multiple targets.

```bash
cargo build --release --target x86_64-linux-android
```

**Important note** : Remember that for the reason ARM and x86 are, by design, different architectures, getting syntax errors while running `cargo test` command on an ARMv7 compatible Rust library with a x86-64 CPU for example is an expected behavior, since the CPU is unable to handle it.

## Setting up Godot

After building the GDNative libraries, we need to link them to Godot, by adding new entries in the GDNative library declaration file (`*.gdnlib`) for `Android.armeabi-v7a` (ARMv7),  `arm64-v8a` (ARM64), `Android.x86` (x86) and/or `Android.x86_64` (x86-64), depending of the toolchains we actually used in previous steps.

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

When we installed Android SDK, it usually comes with :

- an **Android Debug Bridge** (`adb`) executable
- a **Java Runtime Environment** (**JRE**), which comes with :
  - a **JAR Signing and Verification Tool** (`jarsigner`) executable
  - a **Java Keytool** (`keytool`) executable
- a **debug Java keystore** (`debug.keystore`) 

Most of the Android related project specific settings are located in `export_presets.cfg` under a proper Android preset and may be edited at our convenience. These usually include :

- the Java package name (`package/unique_name`)
- the screen orientation (`screen/orientation`)
- the app permissions (`permissions/*`)
- the supported architectures (`architectures/*`)
- the launcher icons (`launcher_icons/*`)
- etc.

The next part is signing the APK that will be generated during the export. Usually, we can choose between releasing an app in _debug_ or _release_ mode. When officially releasing it for Play Store, _release_ mode is required.

If not properly handled, Godot Engine will fail to export the game or Play Protect will consider the app as unsecured.

Godot Engine requires at least the **absolute paths** for `adb`, `jarsigner` and a debug keystore. These can be set directly from the GUI (*Editor > Editor Settings*) or in Godot editor configuration file `editor-settings-3.tres` which can be found on :

- Windows : `AppData\Roaming\Godot\`
- UNIX : `~/.config/godot/`

```
# Example based on Ubuntu
export/android/adb = "/usr/local/lib/android/sdk/platform-tools/adb"
export/android/jarsigner = "/usr/bin/jarsigner"
export/android/debug_keystore = "..." # Path for the debug keystore
export/android/debug_keystore_user = "..." # Alias
export/android/debug_keystore_pass = "..." # Keystore password
```

In _release_ mode, we may instead generate a project-specific Java release keystore and register its path on `export_presets.cfg`, following these steps :

- Generate a release keystore, using `keytool`, and set up an alias and a single password (as related in Godot [official docs](https://docs.godotengine.org/en/3.2/getting_started/workflow/export/exporting_for_android.html#exporting-for-google-play-store), `-storepass` and `-keypass` option values must be the same)

```bash
keytool -genkeypair -v -keystore path/to/my.keystore -alias some-alias -keyalg RSA -keysize 2048 -validity 10000 -storepass my-password -keypass my-password
```

- Register the release keystore in `export_presets.cfg` 

```
# Remember to not commit the password as is in VCS !
keystore/release="path/to/my.keystore"
keystore/release_user="some-alias"
keystore/release_password="my-password"
```

- Export the project using the GUI (_Project > Export... > Our Android preset_) and uncheck "_Export with Debug_" in GUI when being asked to enter APK file name, or use one of the following commands :

```bash
# Debug mode
godot --export-debug "Android" path/to/my.apk

# Release mode
godot --export "Android" path/to/my.apk
```

When directly installing the APK on an Android device, Play Protect may display warning explaining that _the app developers are not recognized, so the app may be unsafe_, this is the expected behavior for an APK in _release_ mode that isn't actually released on Play Store. 

If not planning to release on Play Store, [Google is providing a form](https://support.google.com/googleplay/android-developer/contact/protectappeals) to file an appeal from Play Protect.
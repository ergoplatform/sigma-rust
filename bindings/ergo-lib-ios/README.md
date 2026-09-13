# Swift wrapper for [C bindings](../ergo-lib-c) of ergo‑lib

## Prerequisites

- **Xcode 15+** – install via the Mac App Store.
- **Homebrew** – used to install dependencies such as cbindgen.
- **Rust** via [rustup](https://rustup.rs). Use rustup to manage toolchains.

## Build instructions

### 1 Build the C library

First build the base C library, `ergo‑lib‑c`:

```shell
# from the project root
cargo build --release --features mnemonic_gen -p ergo-lib-c
```

This command produces a static library in `target/release/`. You will need the nightly toolchain for this step because we rely on macros to generate portions of the C bindings; a nightly `rustc` can see through the macros so the compiler is able to expand them before `cbindgen` runs.

Next generate C headers using cbindgen (install it via Homebrew if needed):

```shell
# generate header file for Swift bindings
rustup override set nightly
cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_lib.h
rustup override unset
```

### 2 Build the Swift wrapper

Change into the iOS binding directory and build the Swift wrapper, pointing the linker at the C library:

```shell
cd bindings/ergo-lib-ios
# link against the C library built above
swift build -Xlinker -L../../target/release/
```

To run the tests you need to pass additional linker flags:

```shell
swift test -Xlinker -L../../target/release/ --skip RestNodeApiTests --skip RestNodeApiIntegrationTests
```

The `RestNodeApiTests` assume that you have an Ergo node running on localhost; skip them if that is not the case.

## Rust iOS targets & build steps

Before building for iOS, update your toolchain and add the appropriate iOS targets:

```shell
rustup update
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

Build the library for both the device and the simulator:

```shell
cargo build --target aarch64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release
```

These commands will produce `libergo_lib_ios.a` libraries under `target/aarch64-apple-ios/release/` and `target/aarch64-apple-ios-sim/release/`.

## Building an Xcode 15 project for iOS (iPhone & Simulator)

Make sure `ergo‑lib‑c` and the Swift wrapper are built as described above. Then, from the root of the `sigma‑rust` repository, generate an Xcode project and build it for your desired SDK:

```shell
# generate the Xcode project
swift package generate-xcodeproj

# build for the iOS Simulator (choose arm or intel based on your Mac)
xcodebuild -project ./ErgoLib.xcodeproj \
  -xcconfig ./Config/iPhoneSimulator_{arm|intel}.xcconfig \
  -sdk iphonesimulator

# build for physical devices
xcodebuild -project ./ErgoLib.xcodeproj \
  -xcconfig ./Config/iPhoneOS.xcconfig \
  -sdk iphoneos
```

Open `ErgoLib.xcodeproj` in Xcode. In **Build Settings** set the fields **Base SDK**, **Excluded Architecture** and **Supported Platforms** to the values shown in the accompanying screenshots. Next set **Other Linker Flags** to `-L/absolute/path/to/sigma-rust/target/release` for simulator builds and `-L/absolute/path/to/sigma-rust/aarch64-apple-ios/release` for device builds.

## Creating an XCFramework & integrating with Swift

Once both device and simulator libraries are built, you can create an XCFramework that bundles them together. Run the following in the project root (adjust paths as needed):

```shell
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libergo_lib_ios.a \
  -library target/aarch64-apple-ios-sim/release/libergo_lib_ios.a \
  -output ErgoLib.xcframework
```

Drag the generated `ErgoLib.xcframework` into your Xcode project. In your target’s **Frameworks, Libraries, and Embedded Content** section, change the framework’s embed option to **Always Embed**.

## Troubleshooting

- **Target not found** – make sure you’ve added the targets using `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`.
- **Linker errors** – verify that the `-L` flags passed to `swift build`, `swift test` or Xcode point to the correct `target/release` directories for the compiled Rust libraries.
- **Missing symbols at runtime** – confirm that you’re embedding the XCFramework in your iOS app target and that the framework is signed correctly.

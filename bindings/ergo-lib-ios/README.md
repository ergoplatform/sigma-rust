```markdown
# Swift wrapper for [C bindings](../ergo-lib-c) of ergo-lib.

## Development Environments

- **iOS/macOS**: Follow the instructions below (requires verification on macOS)
- **Linux**: See [Linux Development Guide](LINUX_DEVELOPMENT.md) for Linux-specific setup

## Prerequisites

- Xcode 15 or later (macOS only)
- Rust toolchain with specific targets
- cbindgen 0.28.0 or later

## Build instructions

1. Build `ergo-lib-c`:
```shell
cargo build --release --features rest --features mnemonic_gen -p ergo-lib-c
```

2. Add required targets:
```shell
# For iPhone simulator on Intel macs
rustup target add x86_64-apple-ios

# For iPhone on Apple silicon macs
rustup target add aarch64-apple-ios

# For iPhone simulator on Apple silicon macs
rustup target add aarch64-apple-ios-sim
```

3. Generate C headers:
```shell
cd bindings/ergo-lib-c
rustup override set nightly-2024-01-26
cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_lib.h
rustup override set stable
```

4. Build Swift project:
```shell
cd ../ergo-lib-ios
swift build -Xlinker -L../../target/release/
```

5. Run tests:
```shell
swift test -Xlinker -L../../target/release/ --skip RestNodeApiTests --skip RestNodeApiIntegrationTests
```
Note: `RestNodeApiTests` require an ergo node running on localhost.

### Building Xcode 15 project for iPhone Simulator

1. Ensure `ergo-lib-c` is built (steps 1-3 above)

2. Generate Xcode project:
```shell
cd bindings/ergo-lib-ios
swift package generate-xcodeproj
xcodebuild -project ./ErgoLib.xcodeproj -xcconfig ./Config/iPhoneSimulator_{arm|intel}.xcconfig -sdk iphonesimulator
```
Choose `arm` or `intel` based on your Mac's architecture.

3. Configure Xcode project:
- Open `ErgoLib.xcodeproj`
- Set linker flags:
  - For Intel Macs: `-L/absolute/path/to/sigma-rust/target/release`
  - For Apple Silicon Macs: `-L/absolute/path/to/sigma-rust/target/aarch64-apple-ios-sim/release`

### Building Xcode 15 project for iPhone (iOS)

1. Build ARM64 target:
```shell
rustup target add aarch64-apple-ios
cargo build --release --target=aarch64-apple-ios -p ergo-lib-c
```

2. Generate Xcode project:
```shell
cd bindings/ergo-lib-ios
swift package generate-xcodeproj
xcodebuild -project ./ErgoLib.xcodeproj -xcconfig ./Config/iPhoneOS.xcconfig -sdk iphoneos
```

3. Configure Xcode settings:
- Open Build Settings
- Set:
  - Base SDK: iOS
  - Excluded Architecture: arm64 (for simulator builds)
  - Supported Platforms: iOS
- Set Other Linker Flags: `-L/absolute/path/to/sigma-rust/aarch64-apple-ios/release`

Note: iOS build instructions require verification on macOS with Xcode 15.
```

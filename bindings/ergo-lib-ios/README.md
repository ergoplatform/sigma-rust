<div align="center">

# ErgoLib Swift

### Swift bindings for Ergo Platform blockchain library

[![Swift](https://img.shields.io/badge/Swift-5.2+-orange.svg)](https://swift.org)
[![Platform](https://img.shields.io/badge/platform-iOS%20%7C%20macOS-lightgrey.svg)](https://developer.apple.com)
[![Xcode](https://img.shields.io/badge/Xcode-15+-blue.svg)](https://developer.apple.com/xcode/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../../LICENSE)

[Features](#features) • [Quick Start](#quick-start) • [Installation](#installation) • [Usage](#usage) • [Build from Source](#build-from-source) • [Troubleshooting](#troubleshooting)

</div>

---

## Overview

**ErgoLib Swift** provides native Swift bindings for the [Ergo Platform](https://ergoplatform.org) blockchain library, enabling iOS and macOS developers to build powerful blockchain applications with idiomatic Swift APIs.

Built on top of the robust [ergo-lib](../../ergo-lib) Rust implementation, this library offers type-safe, memory-safe interfaces for:

- **Wallet Operations**: Create, manage, and sign transactions
- **Address Management**: Generate and validate Ergo addresses (P2PK, P2SH, P2S)
- **Transaction Building**: Construct complex multi-input/output transactions
- **ErgoScript**: Interact with Ergo's smart contract language
- **Mnemonic Generation**: BIP39-compliant seed phrase generation
- **Box Selection**: Automatic UTXO selection for transaction inputs
- **REST API Integration**: Connect to Ergo nodes

## Features

### 🔐 **Cryptography & Security**
- BIP39 mnemonic generation and validation (12-24 word phrases)
- HD wallet support (BIP32/BIP44 derivation paths)
- Secure key management with native iOS Keychain integration patterns
- Message signing and verification

### 💼 **Wallet Operations**
- Generate addresses from mnemonics or public keys
- Create and sign transactions
- Multi-signature transaction support
- Token (custom asset) handling

### 🔗 **Blockchain Integration**
- Connect to Ergo mainnet and testnet
- Query blockchain state
- Submit transactions
- Monitor box (UTXO) changes

### 📦 **Transaction Building**
- Fluent API for transaction construction
- Automatic fee calculation
- Box (UTXO) selection strategies
- Support for complex ErgoScript contracts

---

## Quick Start

### 30-Second Example

```swift
import ErgoLib

// Generate a new wallet
let mnemonic = try MnemonicGenerator(language: "english", strength: 256).generate()
print("🔑 Mnemonic: \(mnemonic)")

// Create an address for testnet
let address = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
print("📬 Address: \(address.toBase58(networkPrefix: .Testnet))")

// Verify address type
if address.typePrefix() == .P2Pk {
    print("✅ Valid P2PK address")
}
```

---

## System Requirements

| Requirement | Version |
|------------|---------|
| **Xcode** | 15.0 or later |
| **Swift** | 5.2 or later |
| **iOS** | 13.0+ (for deployment) |
| **macOS** | 11.0+ (for deployment) |
| **Rust** | 1.87 or later |
| **cbindgen** | Latest |

### Verify Your Environment

```shell
# Check Rust version
rustc --version  # Should show 1.87 or higher

# Check Swift version  
swift --version  # Should show 5.2 or higher

# Check Xcode version
xcodebuild -version  # Should show 15.0 or higher
```

---

## Installation

### Option 1: Swift Package Manager (Recommended)

Add to your `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/ergoplatform/sigma-rust.git", .branch("develop"))
],
targets: [
    .target(
        name: "YourApp",
        dependencies: [
            .product(name: "ErgoLib", package: "sigma-rust")
        ]
    )
]
```

**Note**: Pre-built binaries are not currently available. You'll need to [build from source](#build-from-source).

### Option 2: Manual Integration

1. Follow the [Build from Source](#build-from-source) instructions
2. Add the generated `ErgoLib.framework` to your Xcode project
3. Ensure the framework is embedded and signed

---

## Usage

### 1. Generate Mnemonic & Wallet

```swift
import ErgoLib

// Generate 24-word mnemonic (highest security)
let generator = try MnemonicGenerator(language: "english", strength: 256)
let mnemonic = try generator.generate()

// Or generate from specific entropy
let entropy: [UInt8] = [39, 77, 111, 111, 102, 33, 39, 0, 39, 77, 111, 111, 102, 33, 39, 0]
let deterministicMnemonic = try generator.generateFromEntropy(entropy: entropy)
```

### 2. Work with Addresses

```swift
// Testnet address
let testnetAddr = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
print(testnetAddr.toBase58(networkPrefix: .Testnet))

// Mainnet address
let mainnetAddr = try Address(withMainnetAddress: "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA")
print(mainnetAddr.toBase58(networkPrefix: .Mainnet))

// Decode without network validation
let anyAddr = try Address(withBase58Address: "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA")

// Check address type
switch anyAddr.typePrefix() {
case .P2Pk:
    print("Pay-to-Public-Key (P2PK)")
case .Pay2Sh:
    print("Pay-to-Script-Hash (P2SH)")
case .Pay2S:
    print("Pay-to-Script (P2S)")
}
```

### 3. Handle Errors

```swift
do {
    let address = try Address(withTestnetAddress: "invalid_address")
} catch WalletError.walletCError(let reason) {
    print("Error: \(reason)")
} catch {
    print("Unexpected error: \(error)")
}
```

### 4. Advanced Examples

For comprehensive examples, see the [test suite](Tests/ErgoLibTests/):
- [Address handling](Tests/ErgoLibTests/AddressTests.swift)
- [Wallet operations](Tests/ErgoLibTests/WalletTests.swift)
- [Transaction creation](Tests/ErgoLibTests/TransactionTests.swift)
- [Box selection](Tests/ErgoLibTests/BoxSelectorTests.swift)
- [Message signing](Tests/ErgoLibTests/MessageSigningTests.swift)

---

## Build from Source

### Prerequisites Installation

#### 1. Install Rust (if not already installed)

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. Install cbindgen

```shell
cargo install cbindgen
```

#### 3. Install iOS Rust Targets

Choose the targets you need based on your development environment:

```shell
# For iOS devices (required for production)
rustup target add aarch64-apple-ios

# For iOS Simulator on Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-ios-sim

# For iOS Simulator on Intel Macs (legacy)
rustup target add x86_64-apple-ios
```

**💡 Tip**: If you're on Apple Silicon, you'll primarily need `aarch64-apple-ios` and `aarch64-apple-ios-sim`.

### Build Process

#### Step 1: Build the Rust Library

Navigate to the root of the `sigma-rust` repository:

```shell
cd /path/to/sigma-rust
```

**Build for your target platform:**

<details>
<summary><b>Apple Silicon Mac (M1/M2/M3/M4)</b></summary>

**For iOS Simulator:**
```shell
cargo build --release \
  --features rest \
  --features mnemonic_gen \
  -p ergo-lib-c \
  --target aarch64-apple-ios-sim
```

**For iOS Device:**
```shell
cargo build --release \
  --features rest \
  --features mnemonic_gen \
  -p ergo-lib-c \
  --target aarch64-apple-ios
```

</details>

<details>
<summary><b>Intel Mac</b></summary>

**For iOS Simulator:**
```shell
cargo build --release \
  --features rest \
  --features mnemonic_gen \
  -p ergo-lib-c \
  --target x86_64-apple-ios
```

**For iOS Device:**
```shell
cargo build --release \
  --features rest \
  --features mnemonic_gen \
  -p ergo-lib-c \
  --target aarch64-apple-ios
```

</details>

**Build output**: `target/<target-triple>/release/libergo.a`

#### Step 2: Generate C Headers

```shell
cd bindings/ergo-lib-c

# Temporarily switch to nightly for macro expansion
rustup override set nightly

# Generate headers
cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_lib.h

# Switch back to stable
rustup override set stable

cd ../ergo-lib-ios
```

**Output**: `bindings/ergo-lib-c/h/ergo_lib.h`

#### Step 3: Build Swift Package

Point the linker to your compiled library:

```shell
# For Apple Silicon Simulator
swift build -Xlinker -L../../target/aarch64-apple-ios-sim/release/

# For Intel Simulator
swift build -Xlinker -L../../target/x86_64-apple-ios/release/

# For iOS Device
swift build -Xlinker -L../../target/aarch64-apple-ios/release/
```

#### Step 4: Run Tests

```shell
# For Apple Silicon Simulator
swift test \
  -Xlinker -L../../target/aarch64-apple-ios-sim/release/ \
  --skip RestNodeApiTests \
  --skip RestNodeApiIntegrationTests

# Adjust path for other architectures as needed
```

**Note**: REST API tests are skipped unless you have a local Ergo node running.

---

## Building with Xcode

### For iOS Simulator

#### 1️⃣ Build Rust Library

Follow [Step 1](#step-1-build-the-rust-library) above for your architecture.

#### 2️⃣ Generate Xcode Project

```shell
cd bindings/ergo-lib-ios
swift package generate-xcodeproj
```

#### 3️⃣ Build with Command Line

```shell
# Apple Silicon
xcodebuild -project ./ErgoLib.xcodeproj \
  -xcconfig ./Config/iPhoneSimulator_arm.xcconfig \
  -sdk iphonesimulator

# Intel
xcodebuild -project ./ErgoLib.xcodeproj \
  -xcconfig ./Config/iPhoneSimulator_intel.xcconfig \
  -sdk iphonesimulator
```

#### 4️⃣ Configure Linker in Xcode

1. Open `ErgoLib.xcodeproj` in Xcode
2. Select the target → **Build Settings**
3. Search for "Other Linker Flags"
4. Set the value to the **absolute path** of your library:

```
-L/absolute/path/to/sigma-rust/target/aarch64-apple-ios-sim/release
```

![Xcode Linker Settings](xcode_linker_settings.png)

5. Build (⌘+B)

### For iOS Device

#### 1️⃣ Build Rust Library for ARM64

```shell
rustup target add aarch64-apple-ios
cargo build --release \
  --features rest \
  --features mnemonic_gen \
  -p ergo-lib-c \
  --target aarch64-apple-ios
```

#### 2️⃣ Generate & Build Xcode Project

```shell
cd bindings/ergo-lib-ios
swift package generate-xcodeproj
xcodebuild -project ./ErgoLib.xcodeproj \
  -xcconfig ./Config/iPhoneOS.xcconfig \
  -sdk iphoneos
```

#### 3️⃣ Configure Xcode Settings

Open `ErgoLib.xcodeproj` → **Build Settings**:

| Setting | Value |
|---------|-------|
| **Base SDK** | iOS (latest) |
| **Excluded Architectures** | armv7, i386 |
| **Supported Platforms** | iOS |
| **Other Linker Flags** | `-L/absolute/path/to/sigma-rust/target/aarch64-apple-ios/release` |

![Xcode iOS Settings](xcode_ios_settings.png)

---

## Troubleshooting

### 🔴 Linker Error: `cannot find -lergo`

**Cause**: Library not built or wrong path

**Solution**:
1. Verify `libergo.a` exists:
   ```shell
   ls target/aarch64-apple-ios-sim/release/libergo.a
   ```
2. Ensure **Other Linker Flags** points to the correct directory
3. Rebuild the library: `cargo clean && cargo build --release ...`

### 🔴 Module Map Error: `module.modulemap not found`

**Cause**: C headers not generated

**Solution**:
```shell
cd bindings/ergo-lib-c
rustup override set nightly
cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_lib.h
rustup override set stable
```

### 🔴 Architecture Mismatch Errors

**Cause**: Building for wrong target

**Solution**:
1. Check your Mac architecture:
   ```shell
   uname -m  # arm64 = Apple Silicon, x86_64 = Intel
   ```
2. Verify installed targets:
   ```shell
   rustup target list --installed
   ```
3. Build for correct target (see [Build Process](#build-process))

### 🔴 `cbindgen: command not found`

**Solution**:
```shell
cargo install cbindgen
```

### 🔴 Rust Version Too Old

**Solution**:
```shell
rustup update stable
rustc --version  # Should be 1.87+
```

### 🔴 Xcode Version Compatibility

This library is tested with **Xcode 15+**. For older versions:
1. Adjust **Base SDK** in Build Settings
2. Update deployment target
3. Check Swift version compatibility

### 🟡 Clean Build

When in doubt, perform a clean build:

```shell
# Clean Rust build
cargo clean

# Clean Swift build
swift package clean

# Clean Xcode (if applicable)
rm -rf .build DerivedData
```

---

## API Documentation

### Core Types

- **`Address`**: Ergo address handling (P2PK, P2SH, P2S)
- **`MnemonicGenerator`**: BIP39 mnemonic generation
- **`SecretKey`**: Private key operations
- **`Transaction`**: Transaction building and signing
- **`ErgoBox`**: UTXO representation
- **`ErgoTree`**: Smart contract representation

For detailed API documentation, see:
- [Source code](Sources/ErgoLib/)
- [Test examples](Tests/ErgoLibTests/)
- [Main ergo-lib documentation](https://docs.rs/ergo-lib)

---

## Architecture

```
┌─────────────────────────────────────┐
│      Swift Application Layer        │
├─────────────────────────────────────┤
│         ErgoLib Swift (This)        │  ← Swift wrapper
├─────────────────────────────────────┤
│           ergo-lib-c                │  ← C FFI bindings
├─────────────────────────────────────┤
│         ergo-lib (Rust)             │  ← Core implementation
└─────────────────────────────────────┘
```

**Why this architecture?**
- **Safety**: Rust's memory safety + Swift's type safety
- **Performance**: Near-native performance
- **Maintainability**: Core logic in Rust, idiomatic API in Swift
- **Cross-platform**: Same core library for all platforms

---

## Performance Considerations

- **Initial load**: First library load may take 100-200ms
- **Transaction signing**: ~10-50ms depending on complexity
- **Address validation**: <1ms
- **Mnemonic generation**: ~50-100ms (crypto-secure randomness)

**Optimization tips**:
- Reuse `Address` and `SecretKey` objects
- Cache frequently used addresses
- Perform heavy operations on background threads

---

## Related Projects

### Official Bindings
- [ergo-lib-wasm](../ergo-lib-wasm) - WebAssembly (JavaScript/TypeScript)
- [ergo-lib-jni](../ergo-lib-jni) - Java (Android/JVM)
- [ergo-lib-python](../ergo-lib-python) - Python
- [ergo-lib-c](../ergo-lib-c) - C (base for this library)

### Projects Using ErgoLib
- [Ergo Wallet](https://github.com/ErgoWallet) - Official wallet implementations
- [Oracle Core](https://github.com/ergoplatform/oracle-core) - Oracle pool implementation
- [Ergo Headless dApp Framework](https://github.com/Emurgo/ergo-headless-dapp-framework) - dApp development

---

## FAQ

<details>
<summary><b>Can I use this in production?</b></summary>

Yes, but ensure thorough testing. The underlying Rust library is production-ready and used in multiple live applications.

</details>

<details>
<summary><b>What's the minimum iOS version?</b></summary>

iOS 13.0+, though the library itself has no strict iOS version requirements—this depends on your Swift Package Manager and Xcode configuration.

</details>

<details>
<summary><b>Does this support macOS/tvOS/watchOS?</b></summary>

The library is primarily tested on iOS, but macOS is supported. tvOS and watchOS are untested but may work with appropriate Rust target installation.

</details>

<details>
<summary><b>Why do I need nightly Rust?</b></summary>

Only for the `cbindgen` step to expand macros. The library itself compiles with stable Rust.

</details>

<details>
<summary><b>Can I use this with React Native or Flutter?</b></summary>

Not directly. For React Native, use [ergo-lib-wasm](../ergo-lib-wasm). For Flutter, consider platform channels with this library or use [ergo-lib-jni](../ergo-lib-jni) for Android.

</details>

---

## Getting Help

- **📖 Documentation**: [Ergo Platform Docs](https://docs.ergoplatform.com)
- **💬 Discord**: [Ergo Platform Discord](https://discord.gg/kj7s7nb) - `#sigma-rust` channel
- **🐛 Issues**: [GitHub Issues](https://github.com/ergoplatform/sigma-rust/issues)
- **🤝 Contributing**: [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

## Contributing

We welcome contributions! Please see:
1. [CONTRIBUTING.md](../../CONTRIBUTING.md) - Contribution guidelines
2. [CODE_OF_CONDUCT.md](../../CODE_OF_CONDUCT.md) - Community standards
3. [Open Issues](https://github.com/ergoplatform/sigma-rust/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) - Good first issues

### Development Setup

```shell
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/sigma-rust.git
cd sigma-rust

# Follow the build instructions above

# Run tests
cd bindings/ergo-lib-ios
swift test -Xlinker -L../../target/aarch64-apple-ios-sim/release/
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

---

## Acknowledgments

- Ergo Platform team and community
- Rust and Swift communities
- All [contributors](https://github.com/ergoplatform/sigma-rust/graphs/contributors)

---

<div align="center">

**Built with ❤️ for the Ergo ecosystem**

[Website](https://ergoplatform.org) • [Documentation](https://docs.ergoplatform.com) • [GitHub](https://github.com/ergoplatform)

</div>
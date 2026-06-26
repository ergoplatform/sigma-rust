[![Coverage Status](https://coveralls.io/repos/github/ergoplatform/sigma-rust/badge.svg)](https://coveralls.io/github/ergoplatform/sigma-rust)

# Sigma-Rust

Rust implementation of [ErgoScript](https://github.com/ScorexFoundation/sigmastate-interpreter) cryptocurrency scripting language.

See [Architecture](docs/architecture.md) for high-level overview.

## 📦 Crates

| Crate | Description |
|-------|-------------|
| [ergo-lib](https://github.com/ergoplatform/sigma-rust/tree/develop/ergo-lib) | Overarching crate exposing wallet-related features: chain types (transactions, boxes, etc.), JSON serialization, box selection for tx inputs, tx builder and signing. Exports other crates API. |
| [ergotree-interpreter](https://github.com/ergoplatform/sigma-rust/tree/develop/ergotree-interpreter) | ErgoTree interpreter |
| [ergotree-ir](https://github.com/ergoplatform/sigma-rust/tree/develop/ergotree-ir) | ErgoTree IR and serialization |
| [ergoscript-compiler](https://github.com/ergoplatform/sigma-rust/tree/develop/ergoscript-compiler) | ErgoScript compiler |
| [sigma-ser](https://github.com/ergoplatform/sigma-rust/tree/develop/sigma-ser) | Ergo binary serialization primitives |

## 🛠️ Development Setup

To get started with sigma-rust development:

1. **Prerequisites**
   - Rust toolchain (stable): https://rustup.rs
   - Git: https://git-scm.com
   - For iOS bindings: Xcode 15+ and command line tools

2. **Clone and build**
   ```bash
   git clone https://github.com/ergoplatform/sigma-rust.git
   cd sigma-rust
   cargo build --release
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

## 📁 Project Structure

```
sigma-rust/
├── ergo-lib/              # Core library functionality
├── ergotree-interpreter/  # ErgoTree interpreter
├── ergotree-ir/          # ErgoTree IR and serialization
├── ergoscript-compiler/  # ErgoScript compiler
├── sigma-ser/            # Binary serialization
├── bindings/             # Language bindings
│   ├── ergo-lib-wasm/    # WebAssembly bindings
│   ├── ergo-lib-ios/     # iOS/Swift bindings (Xcode 15+)
│   ├── ergo-lib-jni/     # Java bindings
│   ├── ergo-lib-c/       # C bindings
│   └── ergo-lib-python/  # Python bindings
├── docs/                 # Documentation
└── .github/              # GitHub Actions workflows
```

## 🔗 Language Bindings

Sigma-Rust provides bindings for multiple platforms:

- **WebAssembly**: `bindings/ergo-lib-wasm/` - TypeScript/JavaScript bindings
- **iOS/macOS**: `bindings/ergo-lib-ios/` - Swift bindings (requires Xcode 15+)
- **Android/JVM**: `bindings/ergo-lib-jni/` - Java Native Interface bindings
- **C**: `bindings/ergo-lib-c/` - Direct C bindings
- **Python**: `bindings/ergo-lib-python/` - Python bindings via cffi

See each bind directory's README for specific build and usage instructions.

## 📚 Documentation

- [Architecture Overview](docs/architecture.md)
- [API Documentation](https://docs.rs/ergo-lib)
- [Contributing Guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## 🐛 Getting Help

- Join the [Ergo Discord](https://discord.gg/kj7s7nb) and ask questions in `#sigma-rust`
- Check existing issues or open a new one
- For security concerns, see our [Security Policy](SECURITY.md)

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on reporting bugs, suggesting features, and submitting pull requests.

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT license

at your option.

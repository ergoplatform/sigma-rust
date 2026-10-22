# Sigma-Rust 🦀🛡️

A high-performance, formal-methods-focused Rust implementation of the [ErgoScript](https://github.com/ScorexFoundation/sigmastate-interpreter) cryptocurrency scripting language.

Sigma-Rust serves as the foundational library for the Ergo ecosystem, providing the tools necessary for building secure, decentralized applications on the Ergo blockchain.

---

## ⚡ Quick Start

Add `ergo-lib` to your `Cargo.toml`:

```toml
[dependencies]
ergo-lib = "0.24"
```

### Simple Transaction Building

```rust
use ergo_lib::chain::transaction::Transaction;
use ergo_lib::ergotree_ir::chain::address::Address;

fn main() {
    // Generate a new Ergo address
    let address = Address::P2PK(todo!("Your Public Key"));
    println!("My Ergo Address: {}", address.to_base58());
}
```

---

## 🏗️ Core Architecture

The project is split into several specialized crates to ensure modularity and performance:

| Crate | Description |
| :--- | :--- |
| **`ergo-lib`** | The main entryway. Includes wallets, tx building, and signing. |
| **`ergotree-interpreter`** | The ErgoTree execution engine. |
| **`ergoscript-compiler`** | Compiles ErgoScript into ErgoTree bytecode. |
| **`sigma-ser`** | Specialized binary serialization for Ergo types. |

---

## 🌐 WebAssembly (WASM) Support

Sigma-Rust is fully compatible with WASM, enabling high-performance Ergo logic directly in the browser. 

```bash
wasm-pack build ergo-lib-wasm
```

---

## 🚀 Features & Goals

- **Safety First**: Leverages Rust's memory safety and strict type system.
- **Efficiency**: Optimized for low-latency transaction signing and verification.
- **Portability**: Runs on Desktop, Mobile (via C/Java bindings), and Web (WASM).

## 🤝 Community & Support

- **Discord**: [Join Ergo Discord](https://discord.gg/ergo-platform)
- **Documentation**: [Official Ergo Docs](https://docs.ergoplatform.com)
- **Crates.io**: [ergo-lib on Crates.io](https://crates.io/crates/ergo-lib)

---
*Maintained by the Ergo Platform Foundation.*

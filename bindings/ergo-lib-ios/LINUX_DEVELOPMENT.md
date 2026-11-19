# Linux Development Guide for Ergo iOS Bindings

## Prerequisites

- Fedora/RHEL-based system (tested on Fedora 40)
- Toolbox or similar container environment (recommended)
- Rust toolchain
- cbindgen 0.28.0 or later

## Development Environment Setup

1. Create and enter development toolbox:
    ```bash
    toolbox create --distro fedora --release 40 fedora-40
    toolbox enter fedora-40
    ```

2. Install system dependencies:
    ```bash
    sudo dnf install gcc make cmake git pkg-config
    ```

3. Install Rust components:
    ```bash
    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"

    # Install required toolchain and targets
    rustup toolchain install nightly-2024-01-26
    rustup target add aarch64-apple-ios
    rustup target add x86_64-apple-ios

    # Install cbindgen
    cargo install cbindgen
    ```

## Building Components

1. Build ergo-lib-c:
    ```bash
    # From sigma-rust root directory
    cargo build --release --features rest --features mnemonic_gen -p ergo-lib-c
    ```

2. Generate C headers:
    ```bash
    cd bindings/ergo-lib-c
    rustup override set nightly-2024-01-26
    cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_lib.h
    rustup override set stable
    ```

3. Verify build artifacts:
    ```bash
    # Check the generated header file
    ls -l h/ergo_lib.h
    
    # Check the library
    ls -l ../../target/release/libergo.*
    ```

## Limitations

- iOS/Swift components cannot be directly tested on Linux
- Full iOS integration testing must be performed on macOS
- This environment is primarily for development and testing of the C bindings

## Troubleshooting

If you encounter any issues:
1. Ensure all prerequisites are installed
2. Verify you're using the correct Rust toolchain (nightly-2024-01-26)
3. Check that all required targets are installed
4. Verify the paths to build artifacts are correct

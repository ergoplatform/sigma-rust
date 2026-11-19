# Linux Development Setup

This guide covers setting up the development environment for sigma-rust on Linux systems.

## Prerequisites

- Fedora/RHEL-based system (tested on Fedora 40)
- Toolbox installed on your system
- Git

## Toolbox Setup

1. Create and enter a new development toolbox:
    ```bash
    # Create a new toolbox (if not already created)
    toolbox create --distro fedora --release 40 fedora-40

    # Enter the toolbox
    toolbox enter fedora-40

    # Verify you're inside the toolbox
    cat /etc/os-release
    ```

2. Clone the repository (if not already done):
    ```bash
    # Navigate to your preferred directory
    cd ~/GitIt  # or your preferred directory

    # Clone the repository
    git clone https://github.com/ergoplatform/sigma-rust.git
    cd sigma-rust
    ```

## Development Environment Setup

1. Install system dependencies:
    ```bash
    sudo dnf install gcc make cmake git pkg-config
    ```

2. Install Rust and required components:
    ```bash
    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"

    # Add required targets
    rustup target add aarch64-apple-ios
    rustup target add x86_64-apple-ios

    # Install nightly toolchain
    rustup toolchain install nightly-2024-01-26

    # Install cbindgen
    cargo install cbindgen
    ```

3. Build Components:
    ```bash
    # Build ergo-lib-c
    cd bindings/ergo-lib-c
    cargo build --release --all-features

    # Generate C headers
    rustup override set nightly
    cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_lib.h
    rustup override set stable
    ```

## Limitations

- iOS/Swift components cannot be directly tested on Linux
- Full iOS integration testing is performed via GitHub Actions

## Development Workflow

1. Make changes to Rust components
2. Build and test locally using the commands above
3. Commit changes with clear commit messages
4. Push to GitHub where Actions will perform full iOS integration testing

## Troubleshooting

If you encounter any issues:
1. Ensure all prerequisites are installed
2. Verify you're using the correct Rust toolchain
3. Check the GitHub Actions logs for detailed error messages
4. Verify you're working inside the toolbox environment

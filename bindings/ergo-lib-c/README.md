## C bindings for ergo-lib

A thin wrapper for core Rust part of [C bindings](../ergo-lib-c-core)
Does not use Rust types in the API and is suitable for C FFI.

## Build instructions

The following command builds the project:

```
cargo build -p ergo-lib-c
```

The C header file is generated with [cbindgen](https://github.com/eqrion/cbindgen). Run the following
command:
```
cbindgen --config cbindgen.toml --crate ergo-lib-c --output h/ergo_wallet.h
```
**Note that we require a nightly version of rustc just for this command.** This is because we use
macros to generate some C-types and `cbindgen` cannot directly generate types through them. However
`cbindgen` gives us the option to expand macros, which can only be done through a nightly version of
`rustc`. **The crate itself can always be compiled by a stable version of `rustc`.**
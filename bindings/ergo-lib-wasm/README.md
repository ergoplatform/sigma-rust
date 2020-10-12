[![Latest Version](https://img.shields.io/crates/v/ergo-lib-wasm.svg)](https://crates.io/crates/ergo-lib-wasm)
[![Documentation](https://docs.rs/ergo-lib-wasm/badge.svg)](https://docs.rs/crate/ergo-lib-wasm)
 [![Latest version](https://img.shields.io/npm/v/ergo-lib-wasm)](https://www.npmjs.com/package/ergo-lib-wasm)

WebAssembly library with JS/TS bindings for [sigma-rust](https://github.com/ergoplatform/sigma-rust).

## Troubleshooting
### When I build the `ergo-lib-wasm` and include the `pkg` folder as dependency in my app I get "TypeError: TextDecoder is not a constructor".

Make sure webpack plugins `TextDecoder` and `TextEncoder` are enabled. Check the following lines in webpack config:
https://github.com/ergoplatform/sigma-rust/blob/develop/bindings/ergo-lib-wasm/webpack.config.js#L16

### Using with `create-react-app`
CRA does not support WASM. But you can workaround it. You need to override webpack config. Check out -
https://stackoverflow.com/questions/59319775/how-to-use-webassembly-wasm-with-create-react-app/59720645#59720645

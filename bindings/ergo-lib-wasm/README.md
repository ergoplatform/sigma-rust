[![Latest Version](https://img.shields.io/crates/v/ergo-lib-wasm.svg)](https://crates.io/crates/ergo-lib-wasm)
[![Documentation](https://docs.rs/ergo-lib-wasm/badge.svg)](https://docs.rs/crate/ergo-lib-wasm)



WebAssembly library with JS/TS bindings for [sigma-rust](https://github.com/ergoplatform/sigma-rust).

Packages(npm):

- [ergo-lib-wasm-browser](https://www.npmjs.com/package/ergo-lib-wasm-browser) [![Latest version](https://img.shields.io/npm/v/ergo-lib-wasm-browser)](https://www.npmjs.com/package/ergo-lib-wasm-browser)
- [ergo-lib-wasm-nodejs](https://www.npmjs.com/package/ergo-lib-wasm-nodejs) [![Latest version](https://img.shields.io/npm/v/ergo-lib-wasm-nodejs)](https://www.npmjs.com/package/ergo-lib-wasm-nodejs)

## Alpha version
On CI build (job `JS tests`) an alpha build of npm packages is published. The version of the alpha build is comprised of the next minor version with git commit hash suffix (i.e if the current version is `0.12.0` then alpha build will be `0.13.0-alpha-[COMMIT]`) and published with `alpha` tag. See `JS tests` CI job output for details. 

## Test
- Scripts in [ergo-lib-wasm/tests](https://github.com/hanbu97/sigma-rust/tree/develop/bindings/ergo-lib-wasm/tests) will run in both `node js tests` and `browser js tests`.
- Scripts in [ergo-lib-wasm/tests_browser](https://github.com/hanbu97/sigma-rust/tree/develop/bindings/ergo-lib-wasm/tests_browser) will only run in `browser js tests`.

## Troubleshooting
### When I build the `ergo-lib-wasm` and include the `pkg` folder as dependency in my app I get "TypeError: TextDecoder is not a constructor".

Make sure webpack plugins `TextDecoder` and `TextEncoder` are enabled. Check the following lines in webpack config:

``` javascript
new webpack.ProvidePlugin({
      TextDecoder: ['text-encoder', 'TextDecoder'],
      TextEncoder: ['text-encoder', 'TextEncoder']
    })
```

https://github.com/ergoplatform/sigma-rust/blob/develop/bindings/ergo-lib-wasm/webpack.config.js#L16

### Using with `create-react-app`
CRA does not support WASM. But you can workaround it. You need to override webpack config. Check out -
https://stackoverflow.com/questions/59319775/how-to-use-webassembly-wasm-with-create-react-app/59720645#59720645






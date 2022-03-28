// ref. https://github.com/httptoolkit/brotli-wasm/blob/main/index.browser.js

// This makes importing ergo_lib_wasm asynchronous (because of dynamic import).
// This is needed here for Webpack v4 or v5 syncWebAssembly, which don't
// allow synchronous import of WebAssembly from an entrypoint.
// module.exports = import("./pkg.bundler/brotli_wasm.js");
module.exports = import("./pkg/ergo_lib_wasm.js");

// We don't want to do this for _all_ usage, because dynamic import isn't
// supported in older node versions.
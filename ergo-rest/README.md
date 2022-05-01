Ergo node REST API library

### Roadmap:
- Peer management;
- Peer discovery;
- Bindings for Wasm/iOS/Android;

## Changelog
See [CHANGELOG.md](CHANGELOG.md).

## Notes on vendored dependencies

This crate vendors two crates as sub-modules: `wasm-timer` and `reqwest`. We can't have them as
sub-crates of `sigma-rust` due to issues around Github releases.

### `wasm-timer`
`wasm-timer` is a simple fork of the original repo at https://github.com/tomaka/wasm-timer. The
crate has not been updated since August 2020, and has a dependency on an older version of the
`parking_lot` which breaks our WASM build (details here:
https://github.com/Amanieu/parking_lot/issues/269).

All we've done is update crate dependencies, in particualar bumping `parking_lot` to version `0.12`.

### `reqwest`
This exists as a workaround that gives `reqwest` the ability to have request timeouts for the
WASM platform (see https://github.com/seanmonstar/reqwest/issues/1135). Currently timeouts are
only implemented in `reqwest` for non-WASM platforms. However there exists a yet-to-be-merged pull
request (https://github.com/seanmonstar/reqwest/pull/1274) that implements timeouts under WASM.

We vendor the WASM related code of `reqwest` and apply the above mentioned PR on top.
This allows WASM-based dependendents of this crate to have timeouts for requests. Non-WASM users
have access to the underlying `reqwest` crate without any modifications.

## Contributing
See [Contributing](../CONTRIBUTING.md) guide.

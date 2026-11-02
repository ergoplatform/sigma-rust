# Contributing to sigma-rust

Thanks for wanting to contribute! There are many ways to contribute and we
appreciate any level you're willing to do.

To get started checkout issues labeled ["help wanted" and "good first issue"](https://github.com/ergoplatform/sigma-rust/labels/help%20wanted)

## Process
As a heads up, we'll be running your PR through the following CI jobs:
- warnings turned to compile errors
- `cargo test`
- `rustfmt` (we recommend to setup running `rustfmt` on file save)
- `clippy`

## Releasing
When we're ready to release, a project owner should do the following:

- Create(checkout) a release branch (naming convention `release/vX-Y-Z` using the `ergo-lib` version);
- install or update "cargo release" with `cargo install cargo-release --version 1.1.5 --locked`;
- for the prepared stable 0.29.0 release, run `cargo release release --workspace --unpublished --exclude ergo-p2p --exclude ergo-chain-generation -vv`; `release` keeps stable 0.29.0 unchanged, while `minor` would bump it to 0.30.0. Do not exclude `ergo-lib-python`: its canonical tag drives PyPI, while `publish = false` blocks crates.io publication;
- check the output and, only after review, repeat the same command with `--execute` to actually execute it (make sure that you have an API token from `crates.io`, otherwise run `cargo login` first);
- Build and publish npm package (`cd bindings/ergo-lib-wasm && npm run publish-nodejs && npm run publish-browser`);
- Merge release branch into develop
- Merge release branch into master
- Make a github release


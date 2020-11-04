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
- `cargo release minor --dry-run -vv` if you intend to bump a minor version or
  `cargo release major --dry-run -vv` if you intend to bump a major version (add
  `--exclude sigma-ser` if you want to skip `sigma-ser` release);
- check the output and run it without `--dry-run` to actually execute it;
- Build and publish npm package (`cd bindings/ergo-lib-wasm && npm run publish-nodejs && npm run publish-browser`);
- Merge release branch into develop
- Merge release branch into master
- Make a github release


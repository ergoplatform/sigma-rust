# Wasm-timer

**This is a simple fork of the original repo at https://github.com/tomaka/wasm-timer**. The crate has
not been updated since August 2020, and has a dependency on an older version of the `parking_lot`
which breaks our WASM build (details here: https://github.com/Amanieu/parking_lot/issues/269).

All we've done is update crate dependencies, in particualar bumping `parking_lot` to version `0.12`.

extern crate cbindgen;
use std::path::Path;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&crate_dir).join("h/ergo_wallet.h");

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(path);
}

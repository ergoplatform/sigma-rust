// Runs `wasm-pack build <args>` with the build machine's paths remapped in the compiled output.
// rustc records source paths (panic locations) in the .wasm; without remapping they include the
// checkout directory, the Cargo home and the target directory, so the same commit built in two
// places gives two different files. With them remapped, a published package can be rebuilt from
// its tag and compared byte for byte.
//
// The flags go in CARGO_ENCODED_RUSTFLAGS, which Cargo uses in preference to RUSTFLAGS and to any
// `build.rustflags` in a Cargo config file (this repository sets none). RUSTFLAGS already in the
// environment is kept, split on whitespace as Cargo itself splits it.
const { spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

// A directory as given and, if different, its real path: either form may be recorded.
function forms(dir) {
  const given = path.resolve(dir);
  let real = given;
  try {
    real = fs.realpathSync(given);
  } catch (_) {
    // not created yet (e.g. a fresh target directory)
  }
  return real === given ? [given] : [given, real];
}
const remap = (dir, to) => forms(dir).map((p) => `--remap-path-prefix=${p}=${to}`);

const inherited = process.env.CARGO_ENCODED_RUSTFLAGS
  ? process.env.CARGO_ENCODED_RUSTFLAGS.split("\x1f")
  : (process.env.RUSTFLAGS || "").split(/\s+/);
const flags = [
  ...inherited.filter(Boolean),
  ...remap(process.env.CARGO_HOME || path.join(os.homedir(), ".cargo"), "/cargo"),
  ...remap(path.join(__dirname, "..", "..", ".."), "/sigma-rust"),
  ...(process.env.CARGO_TARGET_DIR ? remap(process.env.CARGO_TARGET_DIR, "/target") : []),
];

const result = spawnSync("wasm-pack", ["build", ...process.argv.slice(2)], {
  stdio: "inherit",
  env: { ...process.env, CARGO_ENCODED_RUSTFLAGS: flags.join("\x1f") },
});
if (result.error) throw result.error;
process.exit(result.status === null ? 1 : result.status);

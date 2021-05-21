const fs = require('fs');

const arg = process.argv.slice(2)[0];

const oldPkg = require(`../pkg${arg}/package.json`);

// based on https://raw.githubusercontent.com/Emurgo/cardano-serialization-lib/master/scripts/publish-helper.js

if (oldPkg.name === 'ergo-lib-wasm') {
  oldPkg.name = oldPkg.name + arg;
}

if (arg === '-browser') {
  // due to a bug in wasm-pack, this file is missing from browser builds
  const missingFile = 'ergo_lib_wasm_bg.js';
  if (oldPkg.files.find(entry => entry === missingFile) == null) {
    oldPkg.files.push(missingFile);
  }
}

console.log(oldPkg);
fs.writeFileSync('./package.json', JSON.stringify(oldPkg, null, 2));

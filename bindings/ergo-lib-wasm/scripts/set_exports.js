const fs = require("fs");

// Remove "module" from the generated (old) {package.json}
const {
  module: unusedModule,
  ...oldPackageJson
} = require(`../pkg-browser/package.json`);

const newPackageJson = {
  ...oldPackageJson,
  type: "module",
  exports: {
    ".": "./ergo_lib_wasm",
  },
};

console.log(newPackageJson);

fs.writeFileSync("./package.json", JSON.stringify(newPackageJson, null, 2));

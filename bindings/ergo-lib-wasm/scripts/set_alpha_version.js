const fs = require('fs');

const argSuffix = process.argv.slice(2)[0];
const argBuildVer = process.argv.slice(2)[1];

const oldPkg = require(`../pkg${argSuffix}/package.json`);

// based on https://raw.githubusercontent.com/Emurgo/cardano-serialization-lib/master/scripts/publish-helper.js

oldPkg.version = oldPkg.version + `-alpha-${argBuildVer}`;

console.log(oldPkg);
fs.writeFileSync('./package.json', JSON.stringify(oldPkg, null, 2));

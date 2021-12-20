import {
    ExtSecretKey,
    DerivationPath,
    NetworkAddress,
    NetworkPrefix,
    Mnemonic
} from '../../pkg-nodejs/ergo_lib_wasm';

const secretSeedFromMnemonic = (mnemonic: string): Uint8Array =>
    Mnemonic.to_seed(mnemonic, "");

const masterSecretFromSeed = (seed: Uint8Array) =>
    ExtSecretKey.derive_master(seed);

const deriveSecretKey = (rootSecret: ExtSecretKey, path: DerivationPath) =>
    rootSecret.derive(path); 

const nextPath = (rootSecret: ExtSecretKey, lastPath: DerivationPath) => 
    rootSecret.derive(lastPath).path().next();

const main = () => {
    const mnemonic = "change me do not use me change me do not use me";

    const seed = secretSeedFromMnemonic(mnemonic);
    const rootSecret = masterSecretFromSeed(seed);

    // This is using EIP-3 pathing: m/44'/429'/0'/0/0
    // First param is 0' (account) and the last 0 (address index)
    let changePath = DerivationPath.new(0, new Uint32Array([0]));
    const changeSecretKey = deriveSecretKey(rootSecret, changePath);
    const changePubKey = changeSecretKey.public_key();
    const changeAddress = NetworkAddress.new(NetworkPrefix.Mainnet, changePubKey.to_address());

    console.log(`Change address: ${changeAddress.to_base58()}`);

    // This is currently required becuase the line:
    // `const changeSecretKey = deriveSecretKey(rootSecret, changePath);`
    // Takes ownership of the changePath pointer and frees it so it's null when we get to this point
    changePath = DerivationPath.new(0, new Uint32Array([0]));
    const firstPath = nextPath(rootSecret, changePath);
    console.log(`First derived path: ${firstPath}`);
    
    const firstSecretKey = deriveSecretKey(rootSecret, firstPath);
    const firstPubkey = firstSecretKey.public_key();
    const firstAddress = NetworkAddress.new(NetworkPrefix.Mainnet, firstPubkey.to_address());

    console.log(`First derived address: ${firstAddress.to_base58()}`);
}

main();

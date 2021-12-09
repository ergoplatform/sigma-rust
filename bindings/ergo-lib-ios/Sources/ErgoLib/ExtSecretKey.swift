import Foundation
import ErgoLibC

class ExtSecretKey {
    internal var pointer: ExtSecretKeyPtr

    /// Create ExtSecretKey from secret key bytes, chain code and derivation path
    /// Derivation path should be a string in the form of: m/44/429/acc'/0/addr
    init(secretKeyBytes: [UInt8], chainCodeBytes: [UInt8], derivationPathStr: String) throws {
        var ptr = ExtSecretKeyPtr?
        let error = derivationPathStr.withCString { cs in
            ergo_lib_ext_secret_key_new(secretKeyBytes, chainCodeBytes, cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }

    /// Derive root extended secret key from seed bytes
    init(seedBytes: [UInt8]) throws {
        var ptr = ExtSecretKeyPtr?
        let error = ergo_lib_ext_secret_key_derive_master(secretKeyBytes, chainCodeBytes, cs, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }

    /// Takes ownership of an existing ```ExtSecretKeyPtr```
    internal init(withRawPointer ptr: ExtSecretKeyPtr) {
        self.pointer = ptr
    }

    /// Derive a new extended secret key from the provided index
    /// The index is in the form of soft or hardened indices
    /// For example: 4 or 4' respectively
    func derive(indexStr: String) throws -> ExtSecretKey {
        var derivedKeyPtr?
        let error = indexStr.withCString { cs in
            ergo_lib_ext_secret_key_derive(self.ptr, cs, &derivedKey)
        }
        try checkError(error)
        return ExtSecretKey(withRawPointer: derivedKeyPtr!)
    }

    deinit {
        ergo_lib_ext_secret_key_delete(self.pointer)
    }
}

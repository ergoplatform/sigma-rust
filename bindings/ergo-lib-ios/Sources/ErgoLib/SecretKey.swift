import Foundation
import ErgoLibC

class SecretKey {
    internal var pointer: SecretKeyPtr
    
    init() {
        var ptr: SecretKeyPtr?
        ergo_wallet_secret_key_generate_random(&ptr)
        self.pointer = ptr!
    }
    
    init(fromBytes : [UInt8]) throws {
        self.pointer = try SecretKey.fromBytes(bytes: fromBytes)
    }
    
    internal init(withRawPointer ptr: BlockHeaderPtr) {
        self.pointer = ptr
    }
    
    func getAddress() -> Address {
        var ptr: AddressPtr?
        ergo_wallet_secret_key_get_address(self.pointer, &ptr)
        return Address(withRawPointer: ptr!)
    }
    
    func toBytes() -> [UInt8] {
        var bytes = Array.init(repeating: UInt8(0), count: 32)
        ergo_wallet_secret_key_to_bytes(self.pointer, &bytes)
        return bytes
    }
    
    private static func fromBytes(bytes: [UInt8]) throws -> SecretKeyPtr {
        var ptr: SecretKeyPtr?
        let error = ergo_wallet_secret_key_from_bytes(bytes, &ptr)
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_secret_key_delete(self.pointer)
    }
}

class SecretKeys {
    internal var pointer: SecretKeysPtr
    
    init() {
        var ptr: SecretKeysPtr?
        ergo_wallet_secret_keys_new(&ptr)
        self.pointer = ptr!
    }
    
    func len() -> UInt {
        return ergo_wallet_secret_keys_len(self.pointer)
    }
    
    func get(index: UInt) -> SecretKey? {
        var ptr: SecretKeyPtr?
        let res = ergo_wallet_secret_keys_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return SecretKey(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    func add(secretKey: SecretKey) {
        ergo_wallet_secret_keys_add(secretKey.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_secret_keys_delete(self.pointer)
    }
}

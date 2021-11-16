import Foundation
import ErgoLibC

class SecretKey {
    internal var pointer: SecretKeyPtr
    
    init() throws {
        self.pointer = try SecretKey.generateRandom()
    }
    
    init(fromBytes : [UInt8]) throws {
        self.pointer = try SecretKey.fromBytes(bytes: fromBytes)
    }
    
    func toBytes() throws -> [UInt8] {
        var bytes = Array.init(repeating: UInt8(0), count: 32)
        let error = ergo_wallet_secret_key_to_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    private static func generateRandom() throws -> SecretKeyPtr {
        var ptr: SecretKeyPtr?
        let error = ergo_wallet_secret_key_generate_random(&ptr)
        try checkError(error)
        return ptr!
    }
    
    private static func fromBytes(bytes: [UInt8]) throws -> SecretKeyPtr {
        var ptr: SecretKeyPtr?
        let error = ergo_wallet_secret_key_from_bytes(bytes, UInt(bytes.count), &ptr)
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_secret_key_delete(self.pointer)
    }
}

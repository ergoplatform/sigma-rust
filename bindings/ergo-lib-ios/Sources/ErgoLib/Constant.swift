
import Foundation
import ErgoLibC

class Constant {
    internal var pointer: ConstantPtr
    
    init(withBase16Str: String) throws  {
        var ptr: ConstantPtr?
        let error = withBase16Str.withCString { cs in
            ergo_wallet_constant_from_base16(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withInt32: Int32) {
        var ptr: ConstantPtr?
        ergo_wallet_constant_from_i32(withInt32, &ptr)
        self.pointer = ptr!
    }
    
    init(withInt64: Int64) {
        var ptr: ConstantPtr?
        ergo_wallet_constant_from_i64(withInt64, &ptr)
        self.pointer = ptr!
    }
    
    init(withBytes: [UInt8]) throws {
        var ptr: ConstantPtr?
        let error = ergo_wallet_constant_from_bytes(withBytes, UInt(withBytes.count), &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withECPointBytes: [UInt8]) throws {
        var ptr: ConstantPtr?
        let error = ergo_wallet_constant_from_ecpoint_bytes(withECPointBytes, UInt(withECPointBytes.count), &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withErgoBox: ErgoBox) {
        var ptr: ConstantPtr?
        ergo_wallet_constant_from_ergo_box(withErgoBox.pointer, &ptr)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: ConstantPtr) {
        self.pointer = ptr
    }
    
    func toBase16String() throws -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_wallet_constant_to_base16(self.pointer, &cStr)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    func toInt32() throws -> Int32 {
        let res = ergo_wallet_constant_to_i32(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func toInt64() throws -> Int64 {
        let res = ergo_wallet_constant_to_i64(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func toBytes() throws -> [UInt8] {
        let res = ergo_wallet_constant_bytes_len(self.pointer)
        try checkError(res.error)
        var bytes = Array.init(repeating: UInt8(0), count: Int(res.value))
        let error = ergo_wallet_constant_to_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    func toErgoBox() throws -> ErgoBox {
        var ptr: ErgoBoxPtr?
        let error = ergo_wallet_constant_to_ergo_box(self.pointer, &ptr)
        try checkError(error)
        return ErgoBox(withRawPointer: ptr!)
    }
    
    
    deinit {
        ergo_wallet_constant_delete(self.pointer)
    }
}

extension Constant: Equatable {
    static func ==(lhs: Constant, rhs: Constant) -> Bool {
        ergo_wallet_constant_eq(lhs.pointer, rhs.pointer)
    }
}

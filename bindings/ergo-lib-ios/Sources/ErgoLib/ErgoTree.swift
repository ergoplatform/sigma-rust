import Foundation
import ErgoLibC

class ErgoTree {
    internal var pointer: ErgoTreePtr
    
    init(fromBytes : [UInt8]) throws {
        self.pointer = try ErgoTree.fromBytes(bytes: fromBytes)
    }
    
    init(fromBase16EncodedString : String) throws {
        self.pointer = try ErgoTree.fromBase16EncodedString(bytesStr: fromBase16EncodedString)
    }
    
    func toBytes() throws -> [UInt8] {
        let res = ergo_wallet_ergo_tree_bytes_len(self.pointer)
        try checkError(res.error)
        var bytes = Array.init(repeating: UInt8(0), count: Int(res.value))
        let error = ergo_wallet_ergo_tree_to_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    func toBase16EncodedString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_ergo_tree_to_base16_bytes(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    func constantsLength() throws -> UInt {
        let res = ergo_wallet_ergo_tree_constants_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func getConstant(index: UInt) throws -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_wallet_ergo_tree_get_constant(self.pointer, index, &constantPtr)
        try checkError(res.error)
        if res.is_some {
            return Constant(withPtr: constantPtr!)
        } else {
            return nil
        }
    }
    
    func withConstant(index: UInt, constant: Constant) throws {
        var newErgoTreePtr: ErgoTreePtr?
        let error = ergo_wallet_ergo_tree_with_constant(self.pointer, index, constant.pointer, &newErgoTreePtr)
        try checkError(error)
        
        // Delete the old ErgoTree and point to new instance
        ergo_wallet_ergo_tree_delete(self.pointer)
        self.pointer = newErgoTreePtr!
    }
    
    func toTemplateBytes() throws -> [UInt8] {
        let res = ergo_wallet_ergo_tree_template_bytes_len(self.pointer)
        try checkError(res.error)
        var bytes = Array.init(repeating: UInt8(0), count: Int(res.value))
        let error = ergo_wallet_ergo_tree_template_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    private static func fromBytes(bytes: [UInt8]) throws -> ErgoTreePtr {
        var ptr: ErgoTreePtr?
        let error = ergo_wallet_ergo_tree_from_bytes(bytes, UInt(bytes.count), &ptr)
        try checkError(error)
        return ptr!
    }
    
    private static func fromBase16EncodedString(bytesStr: String) throws -> ErgoTreePtr {
        var ptr: ErgoTreePtr?
        let error = bytesStr.withCString { cs in
            ergo_wallet_ergo_tree_from_base16_bytes(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_ergo_tree_delete(self.pointer)
    }
}


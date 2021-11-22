import Foundation
import ErgoLibC

class BoxId {
    internal var pointer: BoxIdPtr
    
    init(withString str: String) throws {
        self.pointer = try BoxId.fromString(str: str)
    }
    
    internal init(withPtr ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    func toBytes() throws -> [UInt8] {
        var bytes = Array.init(repeating: UInt8(0), count: 32)
        let error = ergo_wallet_box_id_to_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    func toString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_box_id_to_str(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    private static func fromString(str: String) throws -> BoxIdPtr {
        var ptr: BoxIdPtr?
        let error = str.withCString { cs in
            ergo_wallet_box_id_from_str(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_box_id_delete(self.pointer)
    }
}

class BoxValue {
    internal var pointer: BoxValuePtr
    
    init(fromInt64 : Int64) throws {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_box_value_from_i64(fromInt64, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withPtr ptr: BoxValuePtr) {
        self.pointer = ptr
    }
    
    func toInt64() throws -> Int64 {
        let res = ergo_wallet_box_value_as_i64(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    deinit {
        ergo_wallet_box_value_delete(self.pointer)
    }
}


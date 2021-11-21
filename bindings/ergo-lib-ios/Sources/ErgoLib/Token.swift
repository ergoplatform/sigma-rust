import Foundation
import ErgoLibC

class TokenId {
    internal var pointer: TokenIdPtr
    
    init(fromBoxId : BoxId) throws {
        var ptr: TokenIdPtr?
        let error = ergo_wallet_token_id_from_box_id(fromBoxId.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(fromBase16EncodedString : String) throws {
        self.pointer = try TokenId.fromBase16EncodedString(bytesStr: fromBase16EncodedString)
    }
    
    func toBase16EncodedString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_token_id_to_str(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    private static func fromBase16EncodedString(bytesStr: String) throws -> ErgoTreePtr {
        var ptr: ErgoTreePtr?
        let error = bytesStr.withCString { cs in
            ergo_wallet_token_id_from_str(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_token_id_delete(self.pointer)
    }
}


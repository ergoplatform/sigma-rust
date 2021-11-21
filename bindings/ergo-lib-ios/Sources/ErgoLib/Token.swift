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
    
    init(withPtr ptr: TokenIdPtr) {
        self.pointer = ptr
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

class TokenAmount {
    internal var pointer: TokenAmountPtr
    
    init(fromInt64 : Int64) throws {
        var ptr: TokenAmountPtr?
        let error = ergo_wallet_token_amount_from_i64(fromInt64, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withPtr ptr: TokenAmountPtr) {
        self.pointer = ptr
    }
    
    func toInt64() throws -> Int64 {
        let res = ergo_wallet_token_amount_as_i64(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    deinit {
        ergo_wallet_token_amount_delete(self.pointer)
    }
}

class Token {
    internal var pointer: TokenPtr
    
    init(tokenId : TokenId, tokenAmount: TokenAmount) throws {
        var ptr: TokenPtr?
        let error = ergo_wallet_token_new(tokenId.pointer, tokenAmount.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    func getId() throws -> TokenId {
        var tokenIdPtr: TokenIdPtr?
        let error = ergo_wallet_token_get_id(self.pointer, &tokenIdPtr)
        try checkError(error)
        return TokenId(withPtr: tokenIdPtr!)
    }
    
    func getAmount() throws -> TokenAmount {
        var tokenAmountPtr: TokenAmountPtr?
        let error = ergo_wallet_token_get_amount(self.pointer, &tokenAmountPtr)
        try checkError(error)
        return TokenAmount(withPtr: tokenAmountPtr!)
    }
    
    deinit {
        ergo_wallet_token_delete(self.pointer)
    }
}

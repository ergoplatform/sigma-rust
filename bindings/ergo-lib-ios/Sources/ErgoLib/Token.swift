import Foundation
import ErgoLibC
import SwiftyJSON

class TokenId {
    internal var pointer: TokenIdPtr
    
    init(fromBoxId : BoxId) {
        var ptr: TokenIdPtr?
        ergo_wallet_token_id_from_box_id(fromBoxId.pointer, &ptr)
        self.pointer = ptr!
    }
    
    init(withPtr ptr: TokenIdPtr) {
        self.pointer = ptr
    }
    
    init(fromBase16EncodedString : String) throws {
        self.pointer = try TokenId.fromBase16EncodedString(bytesStr: fromBase16EncodedString)
    }
    
    func toBase16EncodedString() -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_wallet_token_id_to_str(self.pointer, &cStr)
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
    
    func toInt64() -> Int64 {
        return ergo_wallet_token_amount_as_i64(self.pointer)
    }
    
    deinit {
        ergo_wallet_token_amount_delete(self.pointer)
    }
}

class Token {
    internal var pointer: TokenPtr
    
    init(tokenId : TokenId, tokenAmount: TokenAmount) {
        var ptr: TokenPtr?
        ergo_wallet_token_new(tokenId.pointer, tokenAmount.pointer, &ptr)
        self.pointer = ptr!
    }
    
    init(withPtr ptr: TokenPtr) {
        self.pointer = ptr
    }
    
    func getId() -> TokenId {
        var tokenIdPtr: TokenIdPtr?
        ergo_wallet_token_get_id(self.pointer, &tokenIdPtr)
        return TokenId(withPtr: tokenIdPtr!)
    }
    
    func getAmount() -> TokenAmount {
        var tokenAmountPtr: TokenAmountPtr?
        ergo_wallet_token_get_amount(self.pointer, &tokenAmountPtr)
        return TokenAmount(withPtr: tokenAmountPtr!)
    }
    
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_token_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_wallet_token_delete(self.pointer)
    }
}

class Tokens {
    internal var pointer: TokensPtr
    
    init() throws {
        var tokensPtr: TokensPtr?
        let error = ergo_wallet_tokens_new(&tokensPtr)
        try checkError(error)
        self.pointer = tokensPtr!
    }
    
    init(withPtr ptr: TokensPtr) {
        self.pointer = ptr
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_tokens_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> Token? {
        var tokenPtr: TokenPtr?
        let res = ergo_wallet_tokens_get(self.pointer, index, &tokenPtr)
        try checkError(res.error)
        if res.is_some {
            return Token(withPtr: tokenPtr!)
        } else {
            return nil
        }
    }
    
    func add(token: Token) throws {
        let error = ergo_wallet_tokens_add(token.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_tokens_delete(self.pointer)
    }
}

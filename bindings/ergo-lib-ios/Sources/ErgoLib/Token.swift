import Foundation
import ErgoLibC

/// Token id (32 byte digest)
class TokenId {
    internal var pointer: TokenIdPtr
    
    /// Create token id from ergo box id (32 byte digest)
    init(fromBoxId : BoxId) {
        var ptr: TokenIdPtr?
        ergo_lib_token_id_from_box_id(fromBoxId.pointer, &ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``TokenIdPtr``. Note: we must ensure that no other instance
    /// of ``TokenId`` can hold this pointer.
    init(withRawPointer ptr: TokenIdPtr) {
        self.pointer = ptr
    }
    
    /// Parse token id (32 byte digest) from base16-encoded string
    init(fromBase16EncodedString : String) throws {
        var ptr: TokenIdPtr?
        let error = fromBase16EncodedString.withCString { cs in
            ergo_lib_token_id_from_str(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Get base16 encoded string
    func toBase16EncodedString() -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_lib_token_id_to_str(self.pointer, &cStr)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    deinit {
        ergo_lib_token_id_delete(self.pointer)
    }
}

extension TokenId: Equatable {
    static func ==(lhs: TokenId, rhs: TokenId) -> Bool {
        ergo_lib_token_id_eq(lhs.pointer, rhs.pointer)
    }
}

/// Token amount with bound checks
class TokenAmount {
    internal var pointer: TokenAmountPtr
    
    /// Create instance from ``Int64`` with bounds check
    init(fromInt64 : Int64) throws {
        var ptr: TokenAmountPtr?
        let error = ergo_lib_token_amount_from_i64(fromInt64, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``TokenAmountPtr``. Note: we must ensure that no other instance
    /// of ``TokenAmount`` can hold this pointer.
    init(withRawPointer ptr: TokenAmountPtr) {
        self.pointer = ptr
    }
    
    /// Get value as ``Int64``
    func toInt64() -> Int64 {
        return ergo_lib_token_amount_as_i64(self.pointer)
    }
    
    deinit {
        ergo_lib_token_amount_delete(self.pointer)
    }
}

extension TokenAmount: Equatable {
    static func ==(lhs: TokenAmount, rhs: TokenAmount) -> Bool {
        ergo_lib_token_amount_eq(lhs.pointer, rhs.pointer)
    }
}

/// Token represented with token id paired with its amount
class Token {
    internal var pointer: TokenPtr
    
    /// Create a token with given token id and amount
    init(tokenId : TokenId, tokenAmount: TokenAmount) {
        var ptr: TokenPtr?
        ergo_lib_token_new(tokenId.pointer, tokenAmount.pointer, &ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``TokenPtr``. Note: we must ensure that no other instance
    /// of ``Token`` can hold this pointer.
    init(withRawPointer ptr: TokenPtr) {
        self.pointer = ptr
    }
    
    /// Get token id
    func getId() -> TokenId {
        var tokenIdPtr: TokenIdPtr?
        ergo_lib_token_get_id(self.pointer, &tokenIdPtr)
        return TokenId(withRawPointer: tokenIdPtr!)
    }
    
    /// Get token amount
    func getAmount() -> TokenAmount {
        var tokenAmountPtr: TokenAmountPtr?
        ergo_lib_token_get_amount(self.pointer, &tokenAmountPtr)
        return TokenAmount(withRawPointer: tokenAmountPtr!)
    }
    
    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_token_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_lib_token_delete(self.pointer)
    }
}

extension Token: Equatable {
    static func ==(lhs: Token, rhs: Token) -> Bool {
        ergo_lib_token_eq(lhs.pointer, rhs.pointer)
    }
}

/// An ordered collection of ``Token``s
class Tokens {
    internal var pointer: TokensPtr
    
    /// Create an empty collection
    init() {
        var tokensPtr: TokensPtr?
        ergo_lib_tokens_new(&tokensPtr)
        self.pointer = tokensPtr!
    }
    
    /// Takes ownership of an existing ``TokensPtr``. Note: we must ensure that no other instance
    /// of ``Tokens`` can hold this pointer.
    init(withPtr ptr: TokensPtr) {
        self.pointer = ptr
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_lib_tokens_len(self.pointer)
    }
    
    /// Returns the ``Token`` at location `index` if it exists.
    func get(index: UInt) -> Token? {
        var tokenPtr: TokenPtr?
        let res = ergo_lib_tokens_get(self.pointer, index, &tokenPtr)
        assert(res.error == nil)
        if res.is_some {
            return Token(withRawPointer: tokenPtr!)
        } else {
            return nil
        }
    }
    
    /// Add a ``Token`` to the end of the collection. Note that the collection has a maximum
    /// capacity of 255 tokens. Will throw error if adding more.
    func add(token: Token) throws {
        let error = ergo_lib_tokens_add(token.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_lib_tokens_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC
import SwiftyJSON

class BlockHeader {
    internal var pointer: BlockHeaderPtr
    
    init(withJson json: String) throws {
        self.pointer = try BlockHeader.fromJSON(json: json)
    }
    
    internal init(withPtr ptr: BlockHeaderPtr) {
        self.pointer = ptr
    }
    
    private static func fromJSON(json: String) throws -> BlockHeaderPtr {
        var blockHeaderPtr: BlockHeaderPtr?
        let error = json.withCString { cs in
            ergo_wallet_block_header_from_json(cs, &blockHeaderPtr)
        }
        try checkError(error)
        return blockHeaderPtr!
    }
    
    deinit { 
        ergo_wallet_block_header_delete(self.pointer)
    }
}

class BlockHeaders {
    internal var pointer: BlockHeadersPtr
    
    init() throws {
        self.pointer = try BlockHeaders.initEmpty()
    }
    
    init(fromJSON: Any) throws {
        let json = JSON(fromJSON)
        if let arr = json.array {
            let headers = try arr.map{try BlockHeader(withJson: $0.stringValue)}
            self.pointer = try BlockHeaders.initEmpty()
            for header in headers {
                try self.add(blockHeader: header)
            }
        } else {
            throw WalletError.walletCError(reason: "BlockHeaders.init(fromJSON): expected [JSON]")
        }
    }
    
    private static func initEmpty() throws -> BlockHeaderPtr {
        var headersPtr: BlockHeadersPtr?
        let error = ergo_wallet_block_headers_new(&headersPtr)
        try checkError(error)
        return headersPtr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_block_headers_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> BlockHeader? {
        var blockHeaderPtr: BlockHeaderPtr?
        let res = ergo_wallet_block_headers_get(self.pointer, index, &blockHeaderPtr)
        try checkError(res.error)
        if res.is_some {
            return BlockHeader(withPtr: blockHeaderPtr!)
        } else {
            return nil
        }
    }
    
    func add(blockHeader: BlockHeader) throws {
        let error = ergo_wallet_block_headers_add(blockHeader.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_block_headers_delete(self.pointer)
    }
}

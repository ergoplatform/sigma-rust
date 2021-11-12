import Foundation
import ErgoLibC

class BlockHeader {
    internal var pointer: BlockHeaderPtr
    
    init(withJson json: String) throws {
        self.pointer = try BlockHeader.fromJSON(json: json)
    }
    
    init(withPtr ptr: BlockHeaderPtr) {
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
        var headersPtr: BlockHeadersPtr?
        let error = ergo_wallet_block_headers_new(&headersPtr)
        try checkError(error)
        self.pointer = headersPtr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_block_headers_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> BlockHeader {
        var blockHeaderPtr: BlockHeaderPtr?
        let error = ergo_wallet_block_headers_get(self.pointer, index, &blockHeaderPtr)
        try checkError(error)
        return BlockHeader(withPtr: blockHeaderPtr!)
    }
    
    func add(blockHeader: BlockHeader) throws {
        let error = ergo_wallet_block_headers_add(blockHeader.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_block_headers_delete(self.pointer)
    }
}

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

extension BlockHeader: Equatable {
    static func ==(lhs: BlockHeader, rhs: BlockHeader) -> Bool {
        ergo_wallet_block_header_eq(lhs.pointer, rhs.pointer)
    }
}

class BlockHeaders {
    internal var pointer: BlockHeadersPtr
    
    init() {
        self.pointer = BlockHeaders.initEmpty()
    }
    
    init(fromJSON: Any) throws {
        let json = JSON(fromJSON)
        if let arr = json.array {
            let headers = try arr.map{try BlockHeader(withJson: $0.stringValue)}
            self.pointer = BlockHeaders.initEmpty()
            for header in headers {
                self.add(blockHeader: header)
            }
        } else {
            throw WalletError.walletCError(reason: "BlockHeaders.init(fromJSON): expected [JSON]")
        }
    }
    
    private static func initEmpty() -> BlockHeaderPtr {
        var headersPtr: BlockHeadersPtr?
        ergo_wallet_block_headers_new(&headersPtr)
        return headersPtr!
    }
    
    func len() -> UInt {
        return ergo_wallet_block_headers_len(self.pointer)
    }
    
    func get(index: UInt) -> BlockHeader? {
        var blockHeaderPtr: BlockHeaderPtr?
        let res = ergo_wallet_block_headers_get(self.pointer, index, &blockHeaderPtr)
        assert(res.error == nil)
        if res.is_some {
            return BlockHeader(withPtr: blockHeaderPtr!)
        } else {
            return nil
        }
    }
    
    func add(blockHeader: BlockHeader) {
        ergo_wallet_block_headers_add(blockHeader.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_block_headers_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC

class BlockHeader {
    internal var pointer: BlockHeaderPtr
    
    init(withJson json: String) throws {
        self.pointer = try BlockHeader.fromJSON(json: json)
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

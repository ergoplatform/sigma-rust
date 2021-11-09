import Foundation
import ErgoLibC

class PreHeader {
    internal var pointer: PreHeaderPtr
    
    init(withBlockHeader blockHeader: BlockHeader) throws {
        self.pointer = try PreHeader.fromBlockHeader(blockHeader: blockHeader)
    }
    
    private static func fromBlockHeader(blockHeader: BlockHeader) throws -> PreHeaderPtr {
        var preHeaderPtr: PreHeaderPtr?
        let error = ergo_wallet_preheader_from_block_header(blockHeader.pointer, &preHeaderPtr)
        try checkError(error)
        return preHeaderPtr!
    }
    
    deinit {
        ergo_wallet_preheader_delete(self.pointer)
    }
}

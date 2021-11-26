import Foundation
import ErgoLibC

class PreHeader {
    internal var pointer: PreHeaderPtr
    
    init(withBlockHeader blockHeader: BlockHeader) {
        var preHeaderPtr: PreHeaderPtr?
        ergo_wallet_preheader_from_block_header(blockHeader.pointer, &preHeaderPtr)
        self.pointer = preHeaderPtr!
    }
    
    deinit {
        ergo_wallet_preheader_delete(self.pointer)
    }
}

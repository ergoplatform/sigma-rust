import Foundation
import ErgoLibC

/// Block header with the current `spendingTransaction`, that can be predicted by a miner before its
/// formation
class PreHeader {
    internal var pointer: PreHeaderPtr
    
    /// Create instance using data from block header
    init(withBlockHeader blockHeader: BlockHeader) {
        var preHeaderPtr: PreHeaderPtr?
        ergo_wallet_preheader_from_block_header(blockHeader.pointer, &preHeaderPtr)
        self.pointer = preHeaderPtr!
    }
    
    deinit {
        ergo_wallet_preheader_delete(self.pointer)
    }
}

extension PreHeader: Equatable {
    static func ==(lhs: PreHeader, rhs: PreHeader) -> Bool {
        ergo_wallet_pre_header_eq(lhs.pointer, rhs.pointer)
    }
}

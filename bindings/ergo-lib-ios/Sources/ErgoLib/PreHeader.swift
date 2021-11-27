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
extension PreHeader: Equatable {
    static func ==(lhs: PreHeader, rhs: PreHeader) -> Bool {
        ergo_wallet_pre_header_eq(lhs.pointer, rhs.pointer)
    }
}

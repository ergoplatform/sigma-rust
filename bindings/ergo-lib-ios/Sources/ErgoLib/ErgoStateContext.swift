
import Foundation
import ErgoLibC

class ErgoStateContext {
    internal var pointer: ErgoStateContextPtr
    
    init(preHeader : PreHeader, headers: BlockHeaders) throws {
        self.pointer = try ErgoStateContext.fromHeaders(preHeader: preHeader, headers: headers)
    }
    
    private static func fromHeaders(preHeader: PreHeader, headers: BlockHeaders) throws -> ErgoStateContextPtr {
        var ergoStateContextPtr: ErgoStateContextPtr?
        let error = ergo_wallet_ergo_state_context_new(preHeader.pointer, headers.pointer, &ergoStateContextPtr)
        try checkError(error)
        return ergoStateContextPtr!
    }
    
    deinit {
        ergo_wallet_ergo_state_context_delete(self.pointer)
    }
}
extension ErgoStateContext: Equatable {
    static func ==(lhs: ErgoStateContext, rhs: ErgoStateContext) -> Bool {
        ergo_wallet_ergo_state_context_eq(lhs.pointer, rhs.pointer)
    }
}

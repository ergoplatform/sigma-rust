
import Foundation
import ErgoLibC

/// Blockchain state (last headers, etc.)
class ErgoStateContext {
    internal var pointer: ErgoStateContextPtr
    
    /// Create new context
    init(preHeader: PreHeader, headers: BlockHeaders, parameters: Parameters) throws {
        var ptr: ErgoStateContextPtr?
        let error = ergo_lib_ergo_state_context_new(preHeader.pointer, headers.pointer, parameters.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    deinit {
        ergo_lib_ergo_state_context_delete(self.pointer)
    }
}
extension ErgoStateContext: Equatable {
    static func ==(lhs: ErgoStateContext, rhs: ErgoStateContext) -> Bool {
        ergo_lib_ergo_state_context_eq(lhs.pointer, rhs.pointer)
    }
}

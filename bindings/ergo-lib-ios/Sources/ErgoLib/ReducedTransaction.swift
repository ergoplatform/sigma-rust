import Foundation
import ErgoLibC

class ReducedTransaction {
    internal var pointer: ReducedTransactionPtr
    
    init(
        unsignedTx: UnsignedTransaction,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes,
        stateContext: ErgoStateContext
    ) throws {
        var ptr: ReducedTransactionPtr?
        let error = ergo_wallet_reduced_tx_from_unsigned_tx(
            unsignedTx.pointer,
            boxesToSpend.pointer,
            dataBoxes.pointer,
            stateContext.pointer,
            &ptr
        )
        try checkError(error)
        self.pointer = ptr!
    }
    
    func getUnsignedTransaction() -> UnsignedTransaction {
        var ptr: UnsignedTransactionPtr?
        ergo_wallet_reduced_tx_unsigned_tx(self.pointer, &ptr)
        return UnsignedTransaction(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_wallet_reduced_tx_delete(self.pointer)
    }
}

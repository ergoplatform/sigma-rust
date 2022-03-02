import Foundation
import ErgoLibC

/// Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input
/// is augmented with ReducedInput which contains a script reduction result.
/// After an unsigned transaction is reduced it can be signed without context.
/// Thus, it can be serialized and transferred for example to Cold Wallet and signed
/// in an environment where secrets are known.
/// see EIP-19 for more details -
/// https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md
class ReducedTransaction {
    internal var pointer: ReducedTransactionPtr
    
    /// Create `reduced` transaction, i.e. unsigned transaction where each unsigned input
    /// is augmented with ReducedInput which contains a script reduction result.
    init(
        unsignedTx: UnsignedTransaction,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes,
        stateContext: ErgoStateContext
    ) throws {
        var ptr: ReducedTransactionPtr?
        let error = ergo_lib_reduced_tx_from_unsigned_tx(
            unsignedTx.pointer,
            boxesToSpend.pointer,
            dataBoxes.pointer,
            stateContext.pointer,
            &ptr
        )
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Returns the unsigned transation
    func getUnsignedTransaction() -> UnsignedTransaction {
        var ptr: UnsignedTransactionPtr?
        ergo_lib_reduced_tx_unsigned_tx(self.pointer, &ptr)
        return UnsignedTransaction(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_lib_reduced_tx_delete(self.pointer)
    }
}

/// Propositions list(public keys)
class Propositions {
    internal var pointer: PropositionsPtr
    
    /// Create empty proposition holder
    init() {
        var ptr: PropositionsPtr?
        ergo_lib_propositions_new(&ptr)
        self.pointer = ptr!
    }
    
    /// Add new proposition
    func addProposition(fromBytes : [UInt8]) throws {
        let error = ergo_lib_propositions_add_proposition_from_bytes(self.pointer, fromBytes, UInt(fromBytes.count))
        try checkError(error)
    }
    
    deinit {
        ergo_lib_propositions_delete(self.pointer)
    }
}

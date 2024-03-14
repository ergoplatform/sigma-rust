
import Foundation
import ErgoLibC

/// A family of hints which are about a correspondence between a public image of a secret image and prover's commitment
/// to randomness ("a" in a sigma protocol).
class CommitmentHint {
    internal var pointer: CommitmentHintPtr
    
    /// Takes ownership of an existing ``CommitmentHint``. Note: we must ensure that no other instance
    /// of ``CommitmentHint`` can hold this pointer.
    internal init(withRawPointer ptr: CommitmentHintPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_lib_commitment_hint_delete(self.pointer)
    }
}

/// Collection of hints to be used by a prover
class HintsBag {
    internal var pointer: HintsBagPtr
    
    /// Create empty ``HintsBag``
    init() {
        var ptr: HintsBagPtr?
        ergo_lib_hints_bag_empty(&ptr)
        self.pointer = ptr!
    }
    
    /// Add commitment hint to the bag
    func addCommitmentHint(hint: CommitmentHint) {
        ergo_lib_hints_bag_add_commitment(self.pointer, hint.pointer)
    }
    
    /// Length of ``HintsBag``
    func len() -> UInt {
        ergo_lib_hints_bag_len(self.pointer)
    }
    
    /// Returns the ``CommitmentHint`` at location `index` if it exists.
    func getCommitmentHint(index: UInt) -> CommitmentHint? {
        var ptr: CommitmentHintPtr?
        let res = ergo_lib_hints_bag_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return CommitmentHint(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    /// Takes ownership of an existing ``HintsBag``. Note: we must ensure that no other instance
    /// of ``HintsBag`` can hold this pointer.
    internal init(withRawPointer ptr: HintsBagPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_lib_hints_bag_delete(self.pointer)
    }
}

/// TransactionHintsBag
class TransactionHintsBag {
    internal var pointer: TransactionHintsBagPtr
    
    /// Create empty ``TransactionHintsBag``
    init() {
        var ptr: TransactionHintsBagPtr?
        ergo_lib_transaction_hints_bag_empty(&ptr)
        self.pointer = ptr!
    }
    
    /// Adding hints for input
    func addHintsForInput(index: UInt, hintsBag: HintsBag) {
        ergo_lib_transaction_hints_bag_add_hints_for_input(self.pointer, index, hintsBag.pointer)
    }
    
    /// Get HintsBag corresponding to input index
    func allHintsForInput(index: UInt) -> HintsBag {
        var ptr: HintsBagPtr?
        ergo_lib_transaction_hints_bag_all_hints_for_input(self.pointer, index, &ptr)
        return HintsBag(withRawPointer: ptr!)
    }
    
    /// Takes ownership of an existing ``TransactionHintsBag``. Note: we must ensure that no other instance
    /// of ``TransactionHintsBag`` can hold this pointer.
    internal init(withRawPointer ptr: TransactionHintsBagPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_lib_transaction_hints_bag_delete(self.pointer)
    }
}

/// Extract hints from signed transaction
func extractHintsFromSignedTransaction(
    transaction: Transaction,
    stateContext: ErgoStateContext,
    boxesToSpend: ErgoBoxes,
    dataBoxes: ErgoBoxes,
    realPropositions: Propositions,
    simulatedPropositions: Propositions
) throws -> TransactionHintsBag {
    var ptr: TransactionHintsBagPtr?
    let error = ergo_lib_transaction_extract_hints(
        transaction.pointer,
        stateContext.pointer,
        boxesToSpend.pointer,
        dataBoxes.pointer,
        realPropositions.pointer,
        simulatedPropositions.pointer,
        &ptr)
    
    try checkError(error)
    return TransactionHintsBag(withRawPointer: ptr!)
}

/// Unsigned (inputs without proofs) transaction
class UnsignedTransaction {
    internal var pointer: UnsignedTransactionPtr
    
    /// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
    /// strings
    init(withJson json: String) throws {
        var ptr: UnsignedTransactionPtr?
        let error = json.withCString { cs in
            ergo_lib_unsigned_tx_from_json(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``UnsignedTransactionPtr``. Note: we must ensure that no other instance
    /// of ``UnsignedTransaction`` can hold this pointer.
    internal init(withRawPointer ptr: UnsignedTransactionPtr) {
        self.pointer = ptr
    }
    
    /// Get ``TxId`` of this unsigned transaction
    func getTxId() -> TxId {
        var ptr: TxIdPtr?
        ergo_lib_unsigned_tx_id(self.pointer, &ptr)
        return TxId(withRawPointer: ptr!)
    }
    
    /// Get ``UnsignedInputs`` for this unsigned transaction
    func getUnsignedInputs() -> UnsignedInputs {
        var unsignedInputsPtr: UnsignedInputsPtr?
        ergo_lib_unsigned_tx_inputs(self.pointer, &unsignedInputsPtr)
        return UnsignedInputs(withRawPointer: unsignedInputsPtr!)
    }
    
    /// Get ``DataInputs`` for this unsigned transaction
    func getDataInputs() -> DataInputs {
        var dataInputsPtr: DataInputsPtr?
        ergo_lib_unsigned_tx_data_inputs(self.pointer, &dataInputsPtr)
        return DataInputs(withRawPointer: dataInputsPtr!)
    }
    
    /// Get output candidates for this unsigned transaction
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_lib_unsigned_tx_output_candidates(self.pointer, &ptr)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_unsigned_tx_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    /// JSON representation according to EIP-12 https://github.com/ergoplatform/eips/pull/23
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_unsigned_tx_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_lib_unsigned_tx_delete(self.pointer)
    }
}

/**
 * ErgoTransaction is an atomic state transition operation. It destroys Boxes from the state
 * and creates new ones. If transaction is spending boxes protected by some non-trivial scripts,
 * its inputs should also contain proof of spending correctness - context extension (user-defined
 * key-value map) and data inputs (links to existing boxes in the state) that may be used during
 * script reduction to crypto, signatures that satisfies the remaining cryptographic protection
 * of the script.
 * Transactions are not encrypted, so it is possible to browse and view every transaction ever
 * collected into a block.
 */
class Transaction {
    internal var pointer: TransactionPtr
    
    /// Create ``Transaction`` from ``UnsignedTransaction`` and an array of proofs in the same order
    /// as `UnsignedTransaction.inputs` with empty proof indicated with empty byte array
    init(unsignedTx: UnsignedTransaction, proofs: ByteArrays) throws {
        var ptr: TransactionPtr?
        let error = ergo_lib_tx_from_unsigned_tx(unsignedTx.pointer, proofs.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
    /// strings
    init(withJson json: String) throws {
        var ptr: TransactionPtr?
        let error = json.withCString { cs in
            ergo_lib_tx_from_json(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``TransactionPtr``. Note: we must ensure that no other instance
    /// of ``Transaction`` can hold this pointer.
    internal init(withRawPointer ptr: TransactionPtr) {
        self.pointer = ptr
    }
    
    /// Get ``TxId`` of this transaction
    func getTxId() -> TxId {
        var ptr: TxIdPtr?
        ergo_lib_tx_id(self.pointer, &ptr)
        return TxId(withRawPointer: ptr!)
    }
    
    /// Get ``Inputs`` for this transaction
    func getInputs() -> Inputs {
        var ptr: UnsignedInputsPtr?
        ergo_lib_tx_inputs(self.pointer, &ptr)
        return Inputs(withRawPointer: ptr!)
    }
    
    /// Get ``DataInputs`` for this transaction
    func getDataInputs() -> DataInputs {
        var ptr: DataInputsPtr?
        ergo_lib_tx_data_inputs(self.pointer, &ptr)
        return DataInputs(withRawPointer: ptr!)
    }
    
    /// Get output candidates for this transaction
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_lib_tx_output_candidates(self.pointer, &ptr)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    /// Returns ``ErgoBoxes`` created from ``ErgoBoxCandidate``'s with tx id and indices
    func getOutputs() -> ErgoBoxes {
        var ptr: ErgoBoxesPtr?
        ergo_lib_tx_outputs(self.pointer, &ptr)
        return ErgoBoxes(withRawPointer: ptr!)
    }
    /// Attempt to validate a transaction. throws an exception if validation fails
    func validateTransaction(
        stateContext: ErgoStateContext,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes
    ) throws {
        let error = ergo_lib_tx_validate(self.pointer, stateContext.pointer, boxesToSpend.pointer, dataBoxes.pointer);
        try checkError(error)
    }

    
    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_tx_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_tx_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_lib_tx_delete(self.pointer)
    }
}

/// Transaction id
class TxId {
    internal var pointer: TxIdPtr
    
    /// Create instance from hex-encoded string
    init(withString str: String) throws {
        var ptr: TxIdPtr?
        let error = str.withCString { cs in
            ergo_lib_tx_id_from_str(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``TxIdPtr``. Note: we must ensure that no other instance
    /// of ``TxId`` can hold this pointer.
    internal init(withRawPointer ptr: TxIdPtr) {
        self.pointer = ptr
    }
    
    /// Get the tx id as bytes
    func toString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_tx_id_to_str(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    deinit {
        ergo_lib_tx_id_delete(self.pointer)
    }
}

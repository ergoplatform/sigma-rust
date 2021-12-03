
import Foundation
import ErgoLibC
import SwiftyJSON

/// Unsigned (inputs without proofs) transaction
class UnsignedTransaction {
    internal var pointer: UnsignedTransactionPtr
    
    /// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
    /// strings
    init(withJson json: String) throws {
        var ptr: UnsignedTransactionPtr?
        let error = json.withCString { cs in
            ergo_wallet_unsigned_tx_from_json(cs, &ptr)
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
        ergo_wallet_unsigned_tx_id(self.pointer, &ptr)
        return TxId(withRawPointer: ptr!)
    }
    
    /// Get ``UnsignedInputs`` for this unsigned transaction
    func getUnsignedInputs() -> UnsignedInputs {
        var unsignedInputsPtr: UnsignedInputsPtr?
        ergo_wallet_unsigned_tx_inputs(self.pointer, &unsignedInputsPtr)
        return UnsignedInputs(withRawPointer: unsignedInputsPtr!)
    }
    
    /// Get ``DataInputs`` for this unsigned transaction
    func getDataInputs() -> DataInputs {
        var dataInputsPtr: DataInputsPtr?
        ergo_wallet_unsigned_tx_data_inputs(self.pointer, &dataInputsPtr)
        return DataInputs(withRawPointer: dataInputsPtr!)
    }
    
    /// Get output candidates for this unsigned transaction
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_unsigned_tx_output_candidates(self.pointer, &ptr)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_unsigned_tx_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    /// JSON representation according to EIP-12 https://github.com/ergoplatform/eips/pull/23
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_unsigned_tx_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_wallet_unsigned_tx_delete(self.pointer)
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
        let error = ergo_wallet_tx_from_unsigned_tx(unsignedTx.pointer, proofs.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Parse from JSON. Supports Ergo Node/Explorer API and box values and token amount encoded as
    /// strings
    init(withJson json: String) throws {
        var ptr: TransactionPtr?
        let error = json.withCString { cs in
            ergo_wallet_tx_from_json(cs, &ptr)
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
        ergo_wallet_tx_id(self.pointer, &ptr)
        return TxId(withRawPointer: ptr!)
    }
    
    /// Get ``Inputs`` for this transaction
    func getInputs() -> Inputs {
        var ptr: UnsignedInputsPtr?
        ergo_wallet_tx_inputs(self.pointer, &ptr)
        return Inputs(withRawPointer: ptr!)
    }
    
    /// Get ``DataInputs`` for this transaction
    func getDataInputs() -> DataInputs {
        var ptr: DataInputsPtr?
        ergo_wallet_tx_data_inputs(self.pointer, &ptr)
        return DataInputs(withRawPointer: ptr!)
    }
    
    /// Get output candidates for this transaction
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_tx_output_candidates(self.pointer, &ptr)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    /// Returns ``ErgoBoxes`` created from ``ErgoBoxCandidate``'s with tx id and indices
    func getOutputs() -> ErgoBoxes {
        var ptr: ErgoBoxesPtr?
        ergo_wallet_tx_outputs(self.pointer, &ptr)
        return ErgoBoxes(withRawPointer: ptr!)
    }
    
    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_tx_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_tx_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_wallet_tx_delete(self.pointer)
    }
}

/// Transaction id
class TxId {
    internal var pointer: TxIdPtr
    
    /// Create instance from hex-encoded string
    init(withString str: String) throws {
        var ptr: TxIdPtr?
        let error = str.withCString { cs in
            ergo_wallet_tx_id_from_str(cs, &ptr)
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
        let error = ergo_wallet_tx_id_to_str(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    deinit {
        ergo_wallet_tx_id_delete(self.pointer)
    }
}

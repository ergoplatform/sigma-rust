
import Foundation
import ErgoLibC
import SwiftyJSON

class UnsignedTransaction {
    internal var pointer: UnsignedTransactionPtr
    
    init(withJson json: String) throws {
        self.pointer = try UnsignedTransaction.fromJSON(json: json)
    }
    
    func getTxId() throws -> TxId {
        var ptr: TxIdPtr?
        let error = ergo_wallet_unsigned_tx_id(self.pointer, &ptr)
        try checkError(error)
        return TxId(withRawPointer: ptr!)
    }
    
    func getUnsignedInputs() throws -> UnsignedInputs {
        var unsignedInputsPtr: UnsignedInputsPtr?
        let error = ergo_wallet_unsigned_tx_inputs(self.pointer, &unsignedInputsPtr)
        try checkError(error)
        return UnsignedInputs(withPtr: unsignedInputsPtr!)
    }
    
    func getDataInputs() throws -> DataInputs {
        var dataInputsPtr: DataInputsPtr?
        let error = ergo_wallet_unsigned_tx_data_inputs(self.pointer, &dataInputsPtr)
        try checkError(error)
        return DataInputs(withPtr: dataInputsPtr!)
    }
    
    func getOutputCandidates() throws -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        let error = ergo_wallet_unsigned_tx_output_candidates(self.pointer, &ptr)
        try checkError(error)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
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
    
    private static func fromJSON(json: String) throws -> UnsignedTransactionPtr {
        var ptr: UnsignedTransactionPtr?
        let error = json.withCString { cs in
            ergo_wallet_unsigned_tx_from_json(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_unsigned_tx_delete(self.pointer)
    }
}

class Transaction {
    internal var pointer: TransactionPtr
    
    init(unsignedTx: UnsignedTransaction, proofs: ByteArrays) throws {
        var ptr: TransactionPtr?
        let error = ergo_wallet_tx_from_unsigned_tx(unsignedTx.pointer, proofs.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withJson json: String) throws {
        self.pointer = try Transaction.fromJSON(json: json)
    }
    
    func getTxId() throws -> TxId {
        var ptr: TxIdPtr?
        let error = ergo_wallet_tx_id(self.pointer, &ptr)
        try checkError(error)
        return TxId(withRawPointer: ptr!)
    }
    
    func getInputs() throws -> Inputs {
        var ptr: UnsignedInputsPtr?
        let error = ergo_wallet_tx_inputs(self.pointer, &ptr)
        try checkError(error)
        return Inputs(withPtr: ptr!)
    }
    
    func getDataInputs() throws -> DataInputs {
        var ptr: DataInputsPtr?
        let error = ergo_wallet_tx_data_inputs(self.pointer, &ptr)
        try checkError(error)
        return DataInputs(withPtr: ptr!)
    }
    
    func getOutputCandidates() throws -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        let error = ergo_wallet_tx_output_candidates(self.pointer, &ptr)
        try checkError(error)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    func getOutputs() throws -> ErgoBoxes {
        var ptr: ErgoBoxesPtr?
        let error = ergo_wallet_tx_outputs(self.pointer, &ptr)
        try checkError(error)
        return ErgoBoxes(withRawPointer: ptr!)
    }
    
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
    
    private static func fromJSON(json: String) throws -> TransactionPtr {
        var ptr: TransactionPtr?
        let error = json.withCString { cs in
            ergo_wallet_tx_from_json(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_tx_delete(self.pointer)
    }
}

class TxId {
    internal var pointer: TxIdPtr
    
    init(withString str: String) throws {
        self.pointer = try TxId.fromString(str: str)
    }
    
    internal init(withRawPointer ptr: TxIdPtr) {
        self.pointer = ptr
    }
    
    func toString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_tx_id_to_str(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    private static func fromString(str: String) throws -> BoxIdPtr {
        var ptr: TxIdPtr?
        let error = str.withCString { cs in
            ergo_wallet_tx_id_from_str(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_tx_id_delete(self.pointer)
    }
}

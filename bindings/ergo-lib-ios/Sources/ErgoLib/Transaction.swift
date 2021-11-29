
import Foundation
import ErgoLibC
import SwiftyJSON

class UnsignedTransaction {
    internal var pointer: UnsignedTransactionPtr
    
    init(withJson json: String) throws {
        self.pointer = try UnsignedTransaction.fromJSON(json: json)
    }
    
    internal init(withRawPointer ptr: BlockHeaderPtr) {
        self.pointer = ptr
    }
    
    func getTxId() -> TxId {
        var ptr: TxIdPtr?
        ergo_wallet_unsigned_tx_id(self.pointer, &ptr)
        return TxId(withRawPointer: ptr!)
    }
    
    func getUnsignedInputs() -> UnsignedInputs {
        var unsignedInputsPtr: UnsignedInputsPtr?
        ergo_wallet_unsigned_tx_inputs(self.pointer, &unsignedInputsPtr)
        return UnsignedInputs(withPtr: unsignedInputsPtr!)
    }
    
    func getDataInputs() -> DataInputs {
        var dataInputsPtr: DataInputsPtr?
        ergo_wallet_unsigned_tx_data_inputs(self.pointer, &dataInputsPtr)
        return DataInputs(withPtr: dataInputsPtr!)
    }
    
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_unsigned_tx_output_candidates(self.pointer, &ptr)
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
    
    func getTxId() -> TxId {
        var ptr: TxIdPtr?
        ergo_wallet_tx_id(self.pointer, &ptr)
        return TxId(withRawPointer: ptr!)
    }
    
    func getInputs() -> Inputs {
        var ptr: UnsignedInputsPtr?
        ergo_wallet_tx_inputs(self.pointer, &ptr)
        return Inputs(withPtr: ptr!)
    }
    
    func getDataInputs() -> DataInputs {
        var ptr: DataInputsPtr?
        ergo_wallet_tx_data_inputs(self.pointer, &ptr)
        return DataInputs(withPtr: ptr!)
    }
    
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_tx_output_candidates(self.pointer, &ptr)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    func getOutputs() -> ErgoBoxes {
        var ptr: ErgoBoxesPtr?
        ergo_wallet_tx_outputs(self.pointer, &ptr)
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


import Foundation
import ErgoLibC
import SwiftyJSON

class UnsignedTransaction {
    internal var pointer: UnsignedTransactionPtr
    
    init(withJson json: String) throws {
        self.pointer = try UnsignedTransaction.fromJSON(json: json)
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

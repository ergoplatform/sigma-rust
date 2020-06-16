import Foundation
import ErgoWalletC

class UnspentInputBoxes {
    private var pointer: UnspentInputBoxesPtr

    init(withJson json: String) throws {
        self.pointer = try UnspentInputBoxes.from_json(json: json)
    }

    private static func from_json(json: String) throws -> UnspentInputBoxesPtr {
        var unspentInputBoxesPtr: UnspentInputBoxesPtr?
        // TODO: handle error
        let _ = json.withCString { cs in
            ergo_wallet_unspent_input_boxes_from_json(cs, &unspentInputBoxesPtr)
        }
        return unspentInputBoxesPtr!
    }

    deinit {
        ergo_wallet_unspent_input_boxes_delete(self.pointer)
    }
}

class Transaction {
    private var pointer: TransactionPtr

    internal init(withRawPointer pointer: TransactionPtr) {
        self.pointer = pointer
    }

    func toJson() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let _ = try ergo_wallet_signed_tx_to_json(self.pointer, &cStr)
        // TODO: process error
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }

    deinit {
        ergo_wallet_delete_signed_tx(self.pointer)
    }
}

struct Wallet {

    // static func new_signed_tx(unspentInputBoxes: UnspentInputBoxes) throws -> Transaction {


    // }
}


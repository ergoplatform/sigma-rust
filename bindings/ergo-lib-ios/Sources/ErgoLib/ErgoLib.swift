import Foundation
import ErgoLibC

enum WalletError: Error {
    case walletCError(reason: String)
}

class UnspentBoxes {
    internal var pointer: UnspentBoxesPtr

    init(withJson json: String) throws {
        self.pointer = try UnspentBoxes.from_json(json: json)
    }

    private static func from_json(json: String) throws -> UnspentBoxesPtr {
        var unspentBoxesPtr: UnspentBoxesPtr?
        let error = json.withCString { cs in
            ergo_wallet_unspent_boxes_from_json(cs, &unspentBoxesPtr)
        }
        try checkError(error)
        return unspentBoxesPtr!
    }

    deinit {
        ergo_wallet_unspent_boxes_delete(self.pointer)
    }
}

internal func checkError(_ error: ErrorPtr?) throws {
    if error == nil {
        return
    }

    let cStringReason = ergo_wallet_error_to_string(error)
    let reason = String(cString: cStringReason!)
    ergo_wallet_delete_string(cStringReason)
    ergo_wallet_delete_error(error)
    throw WalletError.walletCError(reason: reason)
}

class Transaction {
    private var pointer: TransactionPtr

    internal init(withRawPointer pointer: TransactionPtr) {
        self.pointer = pointer
    }

    func toJson() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_signed_tx_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }

    deinit {
        ergo_wallet_delete_signed_tx(self.pointer)
    }
}



import Foundation
import ErgoLibC

enum WalletError: Error {
    case walletCError(reason: String)
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


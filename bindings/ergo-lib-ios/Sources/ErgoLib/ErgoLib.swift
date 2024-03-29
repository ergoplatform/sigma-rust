import Foundation
import ErgoLibC

/// Simple wrapper for errors emitted from `ergo-lib`
enum WalletError: Error {
    case walletCError(reason: String)
}

internal func checkError(_ error: ErrorPtr?) throws {
    if error == nil {
        return
    }

    let cStringReason = ergo_lib_error_to_string(error)
    let reason = String(cString: cStringReason!)
    ergo_lib_delete_string(cStringReason)
    ergo_lib_delete_error(error)
    throw WalletError.walletCError(reason: reason)
}


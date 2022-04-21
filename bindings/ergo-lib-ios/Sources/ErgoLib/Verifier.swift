import Foundation
import ErgoLibC

/// Verify that the signature is presented to satisfy SigmaProp conditions.
func verifySignature(
    address: Address,
    message: [UInt8],
    signature: SignedMessage
) throws -> Bool {
    let res = ergo_lib_verify_signature(
        address.pointer,
        message,
        UInt(message.count),
        signature.pointer
    )
    try checkError(res.error)
    return res.value
}

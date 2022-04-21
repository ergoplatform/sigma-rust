import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class MessageSigningTests: XCTestCase {
    func testSigningAndVerifying() throws {
        let key = SecretKey()
        let addr = key.getAddress()
        let keys = SecretKeys()
        keys.add(secretKey: key)
        let wallet = Wallet(secrets: keys)
        let msg: [UInt8] = Array("this is a message".utf8)
        let sig = try wallet.signedMessageUsingP2PK(address: addr, message: msg)
        XCTAssert(try verifySignature(address: addr, message: msg, signature: sig))
    }
}

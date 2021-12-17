
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class SecreteKeyTests: XCTestCase {
    func testSecretKey() throws {
        let key = SecretKey()
        let bytes = key.toBytes()
        let newKey = try SecretKey(fromBytes: bytes)
        XCTAssertEqual(bytes, newKey.toBytes())
    }
}

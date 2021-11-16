
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class SecreteKeyTests: XCTestCase {
    func testSecretKey() throws {
        let key = try SecretKey()
        let bytes = try key.toBytes()
        let newKey = try SecretKey(fromBytes: bytes)
        XCTAssertEqual(bytes, try newKey.toBytes())
    }
}

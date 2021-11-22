import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class ErgoBoxTests: XCTestCase {
    func testBoxId() throws {
        let str = "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"
        let boxId = try BoxId(withString: str)
        XCTAssertNoThrow(try boxId.toBytes())
        let newStr = try boxId.toString()
        XCTAssertEqual(str, newStr)
    }
    
    func testBoxValue() throws {
        let amount = Int64(12345678)
        let boxValue = try BoxValue(fromInt64: amount)
        XCTAssertEqual(try boxValue.toInt64(), amount)
    }
}


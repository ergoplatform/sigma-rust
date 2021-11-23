
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class TransactionTests: XCTestCase {
    func testTxId() throws {
        let str = "93d344aa527e18e5a221db060ea1a868f46b61e4537e6e5f69ecc40334c15e38"
        let txId = try TxId(withString: str)
        XCTAssertEqual(try txId.toString(), str)
    }
    
    func testUnsignedTransaction() throws {
    }
}

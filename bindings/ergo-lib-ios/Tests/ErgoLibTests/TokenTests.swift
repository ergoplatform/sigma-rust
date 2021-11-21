
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class TokenIdTests: XCTestCase {
    func testTokenIdFromBoxId() throws {
        let str = "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"
        let boxId = try BoxId(withString: str)
        let tokenId = try TokenId(fromBoxId: boxId)
        XCTAssertNoThrow(try tokenId.toBase16EncodedString())
    }
    
    func testTokenIdFromStr() throws {
        let str = "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac"
        let tokenId = try TokenId(fromBase16EncodedString: str )
        XCTAssertEqual(try tokenId.toBase16EncodedString(), str)
    }
    
    func testTokenAmount() throws {
        let amount = Int64(12345678)
        let tokenAmount = try TokenAmount(fromInt64: amount)
        XCTAssertEqual(try tokenAmount.toInt64(), amount)
    }
    
    func testToken() throws {
        let str = "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac"
        let tokenId = try TokenId(fromBase16EncodedString: str )
        let amount = Int64(12345678)
        let tokenAmount = try TokenAmount(fromInt64: amount)
        let token = try Token(tokenId: tokenId, tokenAmount: tokenAmount)
        let newTokenId = try token.getId()
        let newTokenAmount = try token.getAmount()
        XCTAssertEqual(try newTokenId.toBase16EncodedString(), str)
        XCTAssertEqual(try tokenAmount.toInt64(), try newTokenAmount.toInt64())
    }
}

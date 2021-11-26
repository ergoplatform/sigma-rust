
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class TokenIdTests: XCTestCase {
    func testTokenIdFromBoxId() throws {
        let str = "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"
        let boxId = try BoxId(withString: str)
        let tokenId = TokenId(fromBoxId: boxId)
        XCTAssertNoThrow(tokenId.toBase16EncodedString())
    }
    
    func testTokenIdFromStr() throws {
        let str = "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac"
        let tokenId = try TokenId(fromBase16EncodedString: str )
        XCTAssertEqual(tokenId.toBase16EncodedString(), str)
    }
    
    func testTokenAmount() throws {
        let amount = Int64(12345678)
        let tokenAmount = try TokenAmount(fromInt64: amount)
        XCTAssertEqual(tokenAmount.toInt64(), amount)
    }
    
    func testToken() throws {
        let str = "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac"
        let tokenId = try TokenId(fromBase16EncodedString: str )
        let amount = Int64(12345678)
        let tokenAmount = try TokenAmount(fromInt64: amount)
        let token = Token(tokenId: tokenId, tokenAmount: tokenAmount)
        let newTokenId = token.getId()
        let newTokenAmount = token.getAmount()
        XCTAssertEqual(newTokenId.toBase16EncodedString(), str)
        XCTAssertEqual(tokenAmount.toInt64(), newTokenAmount.toInt64())
    }
    
    func testTokens() throws {
        let tokens = Tokens()
        XCTAssertEqual(tokens.len(), UInt(0))
        XCTAssertNil(tokens.get(index: 3))
        let str = "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac"
        let tokenId = try TokenId(fromBase16EncodedString: str )
        let amount = Int64(12345678)
        let tokenAmount = try TokenAmount(fromInt64: amount)
        let token = Token(tokenId: tokenId, tokenAmount: tokenAmount)
        
        for _ in 1...255 {
            try tokens.add(token: token)
        }
        XCTAssertEqual(tokens.len(), UInt(255))
        XCTAssertNotNil(tokens.get(index: 254))
        
        // Add 256th token, expecting error
        XCTAssertThrowsError(try tokens.add(token: token))
    }
}

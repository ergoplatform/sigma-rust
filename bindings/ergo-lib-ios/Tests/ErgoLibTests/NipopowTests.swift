import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class PoPowHeaderTests: XCTestCase {
    // Test deserialization for PoPowHeader by comparing it with manually deserializing the header and interlinks
    func testFromJson() throws {
        // sample PoPowHeader generated from ergo-chain-generation testing infrastructure
        let json = """
          {
            "header": {
            "version": 1,
            "id": "06fa44cc86a2cf45312fb1587f5eed0d9611a984344446e3f21c6e18422fe334",
            "parentId": "0000000000000000000000000000000000000000000000000000000000000000",
            "adProofsRoot": "d6379a936c8f885b347cd3d03f068b41b7d268729843530650348cf0755fea6d",
            "stateRoot": "000000000000000000000000000000000000000000000000000000000000000000",
            "transactionsRoot": "024562cb92738cbe9f5345b6d78b0272f4823692d2a16a11d80459c6d61f7860",
            "timestamp": 0,
            "nBits": 16842752,
            "height": 1,
            "extensionHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
            "powSolutions": {
            "pk": "038b0f29a60fa8d7e1aeafbe512288a6c6bc696547bbf8247db23c95e83014513c",
            "w": "02c80663dc9fcabee47c14a17c774eefda19cbcf2ea59f2dc70c2a544358ecb56b",
            "n": "8000000000000000",
            "d": "106003266939236557704060319805453778767683550618649496709275559630020036173535"
            },
            "votes": "000000"
            },
            "interlinks": []
            }
          """;
        let header = """
          {
            "version": 1,
            "id": "06fa44cc86a2cf45312fb1587f5eed0d9611a984344446e3f21c6e18422fe334",
            "parentId": "0000000000000000000000000000000000000000000000000000000000000000",
            "adProofsRoot": "d6379a936c8f885b347cd3d03f068b41b7d268729843530650348cf0755fea6d",
            "stateRoot": "000000000000000000000000000000000000000000000000000000000000000000",
            "transactionsRoot": "024562cb92738cbe9f5345b6d78b0272f4823692d2a16a11d80459c6d61f7860",
            "timestamp": 0,
            "nBits": 16842752,
            "height": 1,
            "extensionHash": "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8",
            "powSolutions": {
            "pk": "038b0f29a60fa8d7e1aeafbe512288a6c6bc696547bbf8247db23c95e83014513c",
            "w": "02c80663dc9fcabee47c14a17c774eefda19cbcf2ea59f2dc70c2a544358ecb56b",
            "n": "8000000000000000",
            "d": "106003266939236557704060319805453778767683550618649496709275559630020036173535"
            },
            "votes": "000000"
           }
        """
        let popow = try PoPowHeader(withJson: json)
        XCTAssert((try popow.getInterlinks()).len() == 0)
        XCTAssert((try popow.getHeader()) == (try BlockHeader(withJson: header)))
    }
}

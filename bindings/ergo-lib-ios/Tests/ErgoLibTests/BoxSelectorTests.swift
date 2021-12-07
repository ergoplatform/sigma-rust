import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class BoxSelectorTests: XCTestCase {
    func test() throws {
        let jsonStr = """
        {
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }
        """
        let json = [jsonStr.data(using: .utf8, allowLossyConversion: false)!]
        let unspentBoxes = try ErgoBoxes(fromJSON: json)
        let selection = try SimpleBoxSelector()
            .select(
                inputs: unspentBoxes,
                targetBalance: BoxValue(fromInt64: Int64(10000000)),
                targetTokens: Tokens()
            )
        XCTAssertEqual(
            selection.getBoxes().get(index: UInt(0))!.getBoxId(),
            unspentBoxes.get(index: UInt(0))!.getBoxId()
        )
    }
}

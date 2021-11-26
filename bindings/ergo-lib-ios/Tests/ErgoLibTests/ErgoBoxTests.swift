import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class ErgoBoxTests: XCTestCase {
    func testBoxId() throws {
        let str = "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"
        let boxId = try BoxId(withString: str)
        let newStr = boxId.toString()
        XCTAssertEqual(str, newStr)
    }
    
    func testBoxValue() throws {
        let amount = Int64(12345678)
        let boxValue = try BoxValue(fromInt64: amount)
        XCTAssertEqual(boxValue.toInt64(), amount)
    }
    
    static func generateErgoBoxComponents() throws -> (BoxId, BoxValue, ErgoTree, Contract) {
        
        let boxId = try BoxId(withString: "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e")
        let boxValue = try BoxValue(fromInt64: Int64(67500000000))
        let ergoTree = try ErgoTree(fromBase16EncodedString: "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301")
        let contract = Contract(fromErgoTree: ergoTree)
        return (boxId, boxValue, ergoTree, contract)
    }
    
    func testErgoBoxInitializer() throws {
        let (boxId, boxValue, ergoTree, contract) = try ErgoBoxTests.generateErgoBoxComponents()
        let txId = try TxId(withString: "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9")
        let tokens = Tokens()
        let creationHeight = UInt32(284761)
        let ergoBox = try
            ErgoBox(
                boxValue: boxValue,
                creationHeight: creationHeight,
                contract: contract,
                txId: txId,
                index: UInt16(1),
                tokens: tokens
            )
        XCTAssertEqual(ergoBox.getCreationHeight(), creationHeight)
        XCTAssertEqual(ergoBox.getBoxId().toString(), boxId.toString() )
        XCTAssertEqual(ergoBox.getBoxValue().toInt64(), boxValue.toInt64() )
        XCTAssertEqual(try ergoBox.getErgoTree().toBase16EncodedString(), try ergoTree.toBase16EncodedString())
    }
    
    func testErgoBoxJSON() throws {
        let json = """
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
        let ergoBox = try ErgoBox(withJson: json)
        let (boxId, boxValue, ergoTree, _) = try ErgoBoxTests.generateErgoBoxComponents()
        XCTAssertEqual(ergoBox.getCreationHeight(), UInt32(284761))
        XCTAssertEqual(ergoBox.getBoxId().toString(), boxId.toString() )
        XCTAssertEqual(ergoBox.getBoxValue().toInt64(), boxValue.toInt64() )
        XCTAssertEqual(try ergoBox.getErgoTree().toBase16EncodedString(), try ergoTree.toBase16EncodedString())
    }
}


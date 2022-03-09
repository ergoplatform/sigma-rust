import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class ConstantTests: XCTestCase {
    func testI32Roundtrip() throws {
        let c = Constant(withInt32: Int32(999999999))
        let encoded = try c.toBase16String()
        let decodedC = try Constant(withBase16Str: encoded)
        XCTAssertEqual(c, decodedC)
    }
    
    func testI64Roundtrip() throws {
        let c = Constant(withInt64: Int64(9223372036854775807))
        let encoded = try c.toBase16String()
        let decodedC = try Constant(withBase16Str: encoded)
        XCTAssertEqual(c, decodedC)
    }
    
    func testByteArrayRoundtrip() throws {
        let arr = [1, 1, 2, 255].map{ UInt8($0)}
        let c = try Constant(withBytes: arr)
        let encoded = try c.toBase16String()
        let decodedC = try Constant(withBase16Str: encoded)
        XCTAssertEqual(c, decodedC)
    }
    
    func testECPointBytes() throws {
        let str = "02d6b2141c21e4f337e9b065a031a6269fb5a49253094fc6243d38662eb765db00"
        XCTAssertNoThrow(try Constant(withECPointBytes: base16StringToBytes(str)!))
    }
    
    func testErgoBoxRoundtrip() throws {
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
        let c = Constant(withErgoBox: ergoBox)
        let encoded = try c.toBase16String()
        let decodedC = try Constant(withBase16Str: encoded)
        XCTAssertEqual(c, decodedC)
    }
}

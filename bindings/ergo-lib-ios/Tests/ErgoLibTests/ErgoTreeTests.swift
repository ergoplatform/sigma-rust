import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class ErgoTreeTests: XCTestCase {
    
    static func ergoTreeBase16Example() -> String {
        return "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301"
    }
    
    static func ergoTreeExample() throws -> ErgoTree {
        return
          try ErgoTree(fromBase16EncodedString: ErgoTreeTests.ergoTreeBase16Example())
    }
    
    func testEncoding() throws {
        let tree = try ErgoTreeTests.ergoTreeExample()
        XCTAssertEqual(try tree.toBase16EncodedString(), ErgoTreeTests.ergoTreeBase16Example())
        XCTAssertNoThrow(try tree.toBytes())
        XCTAssertNoThrow(try tree.toTemplateBytes())
    }
    func testConstantsLen() throws {
        let tree = try ErgoTreeTests.ergoTreeExample()
        XCTAssertEqual(UInt(2), try tree.constantsLength())
    }
    
    func testGetConstant() throws {
        let tree = try ErgoTreeTests.ergoTreeExample()
        XCTAssertNotNil(try tree.getConstant(index: UInt(0)))
        XCTAssertNotNil(try tree.getConstant(index: UInt(1)))
        XCTAssertNil(try tree.getConstant(index: UInt(2)))
    }
    
    
    func testWithConstant() throws {
        let treeBytesStr = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301"
        let tree = try ErgoTree(fromBase16EncodedString: treeBytesStr)
        XCTAssertEqual(try tree.constantsLength(), UInt(2))
        let constant = Constant(withInt32: Int32(99))
        try tree.withConstant(index: UInt(0), constant: constant)
        XCTAssertEqual(try tree.getConstant(index: UInt(0))!.toInt32(), Int32(99))
    }
    
    func testWithConstantOutOfBounds() throws {
        let treeBytesStr = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301"
        let tree = try ErgoTree(fromBase16EncodedString: treeBytesStr)
        XCTAssertEqual(try tree.constantsLength(), UInt(2))
        let constant = Constant(withInt32: Int32(99))
        XCTAssertThrowsError(try tree.withConstant(index: UInt(3), constant: constant))
    }
    
    func testWithConstantTypeMismatch() throws {
        let treeBytesStr = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301"
        let tree = try ErgoTree(fromBase16EncodedString: treeBytesStr)
        XCTAssertEqual(try tree.constantsLength(), UInt(2))
        let constant = Constant(withInt64: Int64(324234))
        XCTAssertThrowsError(try tree.withConstant(index: UInt(0), constant: constant))
    }
}


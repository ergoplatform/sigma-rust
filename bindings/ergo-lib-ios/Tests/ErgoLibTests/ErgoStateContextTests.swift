import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class ErgoStateContextTests: XCTestCase {
    func testErgoStateContext() throws {
        let headerJSON = HeaderTests.jsonHeaderExample()
        let blockHeader = try BlockHeader(withJson: headerJSON)
        let preHeader = PreHeader(withBlockHeader: blockHeader)
        let parameters = Parameters()
        var blockHeadersJSON = Array(repeating: HeaderTests.jsonHeaderExample(), count: 10)
        var blockHeaders = try BlockHeaders(fromJSON: blockHeadersJSON)
        XCTAssertNoThrow(try ErgoStateContext(preHeader: preHeader, headers: blockHeaders, parameters: parameters))
        
        // Now test for incorrect number of block headers
        blockHeadersJSON = Array(repeating: HeaderTests.jsonHeaderExample(), count: 8)
        blockHeaders = try BlockHeaders(fromJSON: blockHeadersJSON)
        XCTAssertThrowsError(try ErgoStateContext(preHeader: preHeader, headers: blockHeaders, parameters: parameters))
    }
    
}

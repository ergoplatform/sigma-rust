
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiAsyncTests: XCTestCase {

    func testGetInfo() throws {
        let expectation = self.expectation(description: "getInfo")
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApiAsync = try RestNodeApiAsync()
        try restNodeApiAsync.getInfo(nodeConf: nodeConf, closureSuccess: { (nodeInfo: NodeInfo) -> () in 
                XCTAssertEqual(nodeInfo.getName(), "ergo-mainnet-4.0.16.1")
                expectation.fulfill()
            }, closureFail: { (e: String) -> () in
                XCTFail(e)
                expectation.fulfill()
            })
        waitForExpectations(timeout: 5, handler: nil)
    }
}

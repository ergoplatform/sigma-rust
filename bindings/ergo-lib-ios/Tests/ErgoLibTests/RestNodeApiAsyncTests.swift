
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiAsyncTests: XCTestCase {

    func testGetInfo() throws {
        let expectation = self.expectation(description: "getInfo")
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApiAsync = try RestNodeApiAsync()
        let _ = try restNodeApiAsync.getInfo(nodeConf: nodeConf,
            closureSuccess: { (nodeInfo: NodeInfo) -> () in 
                XCTAssert(!nodeInfo.getName().isEmpty)
                expectation.fulfill()
            }, closureFail: { (e: String) -> () in
                XCTFail(e)
                expectation.fulfill()
            })
        waitForExpectations(timeout: 5, handler: nil)
    }

    func testGetInfoAbort() throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApiAsync = try RestNodeApiAsync()
        let handle = try restNodeApiAsync.getInfo(nodeConf: nodeConf,
            closureSuccess: { (nodeInfo: NodeInfo) -> () in 
                XCTAssert(!nodeInfo.getName().isEmpty)
            }, closureFail: { (e: String) -> () in
                XCTFail(e)
            })
        handle.abort()
    }
}

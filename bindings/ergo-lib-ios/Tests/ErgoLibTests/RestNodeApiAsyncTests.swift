
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiAsyncTests: XCTestCase {

    func testGetInfo() throws {
        let expectation = self.expectation(description: "getInfo")
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApiAsync = try RestNodeApiAsync()
        let _ = try restNodeApiAsync.getInfo(nodeConf: nodeConf,
            closure: { (res: Result<NodeInfo, Error>) -> () in 
                switch res {
                    case .success(let nodeInfo): 
                        XCTAssert(!nodeInfo.getName().isEmpty)
                    case .failure(let error): 
                        XCTFail(error.localizedDescription)
                }
                expectation.fulfill()
            })
        waitForExpectations(timeout: 5, handler: nil)
    }

    func testGetInfoAbort() throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApiAsync = try RestNodeApiAsync()
        let handle = try restNodeApiAsync.getInfo(nodeConf: nodeConf,
            closure: { (res: Result<NodeInfo, Error>) -> () in })
        handle.abort()
    }
}

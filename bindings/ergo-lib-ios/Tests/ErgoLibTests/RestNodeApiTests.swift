
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiTests: XCTestCase {

    func testGetInfo() throws {
        let expectation = self.expectation(description: "getInfo")
        let nodeConf = try NodeConf(withAddrString: "127.0.0.1:9053")
        let restNodeApi = try RestNodeApi()
        let _ = try restNodeApi.getInfo(nodeConf: nodeConf,
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
        let nodeConf = try NodeConf(withAddrString: "127.0.0.1:9053")
        let restNodeApi = try RestNodeApi()
        let handle = try restNodeApi.getInfo(nodeConf: nodeConf,
            closure: { (res: Result<NodeInfo, Error>) -> () in 
                XCTFail("this should not be called")
            })
        handle.abort()
    }

    // need macOS 12.0 on Github GA
     func testGetInfoAsync() throws {
         let nodeConf = try NodeConf(withAddrString: "127.0.0.1:9053")
         let restNodeApi = try RestNodeApi()
         XCTAssertNoThrow(Task(priority: .medium) {
             let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf)
             XCTAssert(!nodeInfo.getName().isEmpty)
         })
         // test of re-using of tokio runtime
         XCTAssertNoThrow(Task(priority: .medium) {
             let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf)
             XCTAssert(!nodeInfo.getName().isEmpty)
         }) 
     } 
}

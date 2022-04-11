
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
    /* func testGetInfoAsync() throws { */
    /*     let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053") */
    /*     let restNodeApi = try RestNodeApi() */
    /*     XCTAssertNoThrow(Task(priority: .medium) { */
    /*         let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf) */
    /*         XCTAssert(!nodeInfo.getName().isEmpty) */
    /*     }) */
    /*     // test of re-using of tokio runtime */
    /*     XCTAssertNoThrow(Task(priority: .medium) { */
    /*         let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf) */
    /*         XCTAssert(!nodeInfo.getName().isEmpty) */
    /*     }) */
    /* } */
    
    func testGetNipopowProofByHeaderId() throws {
        let expectation = self.expectation(description: "getNipopowByHeaderIdAsync")
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApi = try RestNodeApi()
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let _ = try restNodeApi.getNipopowProofByHeaderId(
            nodeConf: nodeConf,
            minChainLength: UInt32(3),
            suffixLen: UInt32(2),
            headerId: blockHeaders.get(index: UInt(0))!.getBlockId(),
            closure: { (res: Result<NipopowProof, Error>) -> () in
                switch res {
                    case .success(_):
                        break
                    case .failure(let error):
                        XCTFail(error.localizedDescription)
                }
                expectation.fulfill()
            })
        waitForExpectations(timeout: 5, handler: nil)
    }
    
    func testGetNipopowProofByHeaderAbort() throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApi = try RestNodeApi()
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let handle = try restNodeApi.getNipopowProofByHeaderId(
            nodeConf: nodeConf,
            minChainLength: UInt32(3),
            suffixLen: UInt32(2),
            headerId: blockHeaders.get(index: UInt(0))!.getBlockId(),
            closure: { (res: Result<NipopowProof, Error>) -> () in
                XCTFail("this should not be called")
            })
        handle.abort()
    }
    
    func testGetNipopowProofByHeaderIdAsync() throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApi = try RestNodeApi()
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        XCTAssertNoThrow(Task(priority: .medium) {
            let proof = try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(3),
                suffixLen: UInt32(2),
                headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
            )
            let _ = try proof.toJSON()!
        })
        // test of re-using of tokio runtime
        XCTAssertNoThrow(Task(priority: .medium) {
            let proof = try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(3),
                suffixLen: UInt32(2),
                headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
            )
            let _ = try proof.toJSON()!
        })
    }
}

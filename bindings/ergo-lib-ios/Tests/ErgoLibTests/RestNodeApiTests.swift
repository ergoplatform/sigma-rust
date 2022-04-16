import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiTests: XCTestCase {
    func testGetNipopowProofByHeaderIdNonAsync() throws {
        let expectation = self.expectation(description: "getNipopowByHeaderIdNonAsync")
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
    
    func testGetNipopowProofByHeaderIdAsync() async throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApi = try RestNodeApi()
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let proof = try await restNodeApi.getNipopowProofByHeaderIdAsync(
            nodeConf: nodeConf,
            minChainLength: UInt32(3),
            suffixLen: UInt32(2),
            headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
        )
        XCTAssertNoThrow(try proof.toJSON()!)
        
        // test of re-using of tokio runtime
        let proofNew = try await restNodeApi.getNipopowProofByHeaderIdAsync(
            nodeConf: nodeConf,
            minChainLength: UInt32(3),
            suffixLen: UInt32(2),
            headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
        )
        XCTAssertNoThrow(try proofNew.toJSON()!)
    }
    
    func testPeerDiscoveryNonAsync() throws {
        let expectation = self.expectation(description: "peerDiscovery")
        let restNodeApi = try RestNodeApi()
        let _ = try restNodeApi.peerDiscovery(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(10),
            timeoutSec: UInt32(1),
            closure: { (res: Result<CStringCollection, Error>) -> () in
                switch res {
                    case .success(let peers):
                        XCTAssertEqual(peers.toArray().count, 0)
                        break
                    case .failure(let error):
                        XCTFail(error.localizedDescription)
                }
                expectation.fulfill()
            })
        waitForExpectations(timeout: 10, handler: nil)
    }
    
    func testPeerDiscoveryAsync() async throws {
        let restNodeApi = try RestNodeApi()
        let peers = try await restNodeApi.peerDiscoveryAsync(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(4),
            timeoutSec: UInt32(1)
        )
        XCTAssert(peers.isEmpty)
        XCTAssertEqual(peers.count, 0)
        
        // test of re-using of tokio runtime
        let peersNew = try await restNodeApi.peerDiscoveryAsync(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(10),
            timeoutSec: UInt32(1)
        )
        XCTAssert(peersNew.isEmpty)
        XCTAssertEqual(peersNew.count, 0)
    }
}

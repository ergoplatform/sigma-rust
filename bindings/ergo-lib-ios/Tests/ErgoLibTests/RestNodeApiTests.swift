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
            closure: { (res: Result<NipopowProof, Error>) -> Void in
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
            closure: { (res: Result<NipopowProof, Error>) -> Void in
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
            maxParallelReqs: UInt16(30),
            timeoutSec: UInt32(3),
            closure: { (res: Result<CStringCollection, Error>) -> Void in
                switch res {
                case .success(let peers):
                    XCTAssert(peers.getLength() > 0)
                    break
                case .failure(let error):
                    XCTFail(error.localizedDescription)
                }
                expectation.fulfill()
            })
        waitForExpectations(timeout: 60, handler: nil)
    }

    func testPeerDiscoveryAsync() async throws {
        let restNodeApi = try RestNodeApi()
        let peers = try await restNodeApi.peerDiscoveryAsync(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(30),
            timeoutSec: UInt32(3)
        )

        XCTAssert(!peers.isEmpty)

        // test of re-using of tokio runtime
        let peersNew = try await restNodeApi.peerDiscoveryAsync(
            seeds: getSeeds(),
            maxParallelReqs: UInt16(30),
            timeoutSec: UInt32(3)
        )
        XCTAssert(!peersNew.isEmpty)
    }

    func testSPVWorkflow() async throws {
        let env = ProcessInfo.processInfo.environment
        if env["CI"] != nil && env["ERGO_SPV_WORKFLOW_TEST"] == nil {
            throw XCTSkip("SPV workflow test requires public ergo nodes; set ERGO_SPV_WORKFLOW_TEST=1 (and optionally ERGO_SPV_PROOF_NODE_1, ERGO_SPV_PROOF_NODE_2, ERGO_SPV_VERIFY_NODE) to run in CI")
        }

        let proofNode1Str = env["ERGO_SPV_PROOF_NODE_1"] ?? "http://159.65.11.55:9053"
        let proofNode2Str = env["ERGO_SPV_PROOF_NODE_2"] ?? "http://76.22.82.80:9053"
        let verifyNodeRaw = env["ERGO_SPV_VERIFY_NODE"] ?? "76.22.82.80:9053"

        let proofNode1 = URL(string: proofNode1Str)!
        let proofNode2 = URL(string: proofNode2Str)!
        let verifyNodeAddrString: String = {
            if verifyNodeRaw.contains("://"),
               let url = URL(string: verifyNodeRaw),
               let host = url.host {
                let port = url.port ?? 9053
                return "\(host):\(port)"
            }
            return verifyNodeRaw
        }()
        let headerId = try BlockId(
            withString: "d1366f762e46b7885496aaab0c42ec2950b0422d48aec3b91f45d4d0cdeb41e5")
        let txId = try TxId(
            withString: "258ddfc09b94b8313bca724de44a0d74010cab26de379be845713cc129546b78")
        let proofs = try await withThrowingTaskGroup(of: [NipopowProof].self) {
            group -> [NipopowProof] in
            group.addTask {
                let proof = try await getNipopowProof(
                    url: proofNode1, headerId: headerId)!
                return [proof]
            }
            group.addTask {
                let proof = try await getNipopowProof(
                    url: proofNode2, headerId: headerId)!
                return [proof]
            }
            return try await group.reduce(into: [NipopowProof]()) { $0 += $1 }
        }

        let genesisBlockId = try BlockId(
            withString: "b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b")
        let verifier = NipopowVerifier(withGenesisBlockId: genesisBlockId)
        try verifier.process(newProof: proofs[0])
        try verifier.process(newProof: proofs[1])
        let bestProof = verifier.bestProof()
        XCTAssertEqual(try bestProof.suffixHead().getHeader().getBlockId(), headerId)

        // Now verify with 3rd node
        let nodeConf = try NodeConf(withAddrString: verifyNodeAddrString)
        let restNodeApi = try RestNodeApi()
        let header = try await restNodeApi.getHeaderAsync(nodeConf: nodeConf, blockId: headerId)
        let merkleProof = try await restNodeApi.getBlocksHeaderIdProofForTxIdAsync(
            nodeConf: nodeConf, blockId: headerId, txId: txId)
        XCTAssert(try merkleProof.valid(expected_root: header.getTransactionsRoot()))
    }
}

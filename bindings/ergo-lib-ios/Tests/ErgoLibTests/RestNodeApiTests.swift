import Foundation
import XCTest

@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiTests: XCTestCase {
    private final class CallbackResultState<Value> {
        private enum State {
            case pending
            case completed(Result<Value, Error>)
            case closed
        }

        private let lock = NSLock()
        private var state = State.pending

        @discardableResult
        func complete(
            _ result: Result<Value, Error>,
            expectation: XCTestExpectation
        ) -> Bool {
            lock.lock()
            defer { lock.unlock() }
            guard case .pending = state else { return false }
            state = .completed(result)
            expectation.fulfill()
            return true
        }

        func closeAndTakeResult() -> Result<Value, Error>? {
            lock.lock()
            defer { lock.unlock() }
            switch state {
            case .pending:
                state = .closed
                return nil
            case .completed(let result):
                state = .closed
                return result
            case .closed:
                return nil
            }
        }
    }

    private enum SyntheticNodeError: Error, LocalizedError {
        case unavailable(URL)

        var errorDescription: String? {
            switch self {
            case .unavailable(let url):
                return "synthetic failure at \(url.absoluteString)"
            }
        }
    }

    private func syntheticExternalNodeReason(for url: URL) -> String {
        "ReqwestError(reqwest::Error { kind: Request, source: "
            + "ConnectError(\"synthetic failure at \(url.absoluteString)\") })"
    }

    private func syntheticExternalNodeError(for url: URL) -> RestNodeApiError {
        RestNodeApiError.misc(syntheticExternalNodeReason(for: url))
    }

    func testFirstMainnetResultTriesUrlsInOrderUntilSuccess() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        var attemptedUrls: [URL] = []

        let result: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
            attemptedUrls.append(url)
            if url == urls[0] {
                throw syntheticExternalNodeError(for: url)
            }
            return 42
        }

        XCTAssertEqual(result, 42)
        XCTAssertEqual(attemptedUrls, urls)
    }

    func testFirstMainnetResultThrowsExplicitFallbackError() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        var attemptedUrls: [URL] = []

        do {
            let _: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
                attemptedUrls.append(url)
                throw syntheticExternalNodeError(for: url)
            }
            XCTFail("Expected every synthetic node attempt to fail")
        } catch let error as MainnetNodeFallbackError {
            XCTAssertEqual(attemptedUrls, urls)
            XCTAssertEqual(error.attemptedUrl, urls[1])
            XCTAssertEqual(
                error.underlyingErrorDescription,
                syntheticExternalNodeReason(for: urls[1])
            )
        } catch {
            XCTFail("Expected MainnetNodeFallbackError, got \(error)")
        }
    }

    func testFirstMainnetResultDoesNotRetryLocalError() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        var attemptedUrls: [URL] = []

        do {
            let _: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
                attemptedUrls.append(url)
                throw SyntheticNodeError.unavailable(url)
            }
            XCTFail("Expected the local error to be rethrown")
        } catch SyntheticNodeError.unavailable(let url) {
            XCTAssertEqual(url, urls[0])
            XCTAssertEqual(attemptedUrls, [urls[0]])
        } catch {
            XCTFail("Expected the unchanged local error, got \(error)")
        }
    }

    func testFirstMainnetResultDoesNotRetryDecodeError() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        let decodeReason = "ReqwestError(reqwest::Error { kind: Decode, "
            + "source: Error(\"request timed out while decoding\") })"
        var attemptedUrls: [URL] = []

        do {
            let _: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
                attemptedUrls.append(url)
                throw RestNodeApiError.misc(decodeReason)
            }
            XCTFail("Expected the decode error to be rethrown")
        } catch RestNodeApiError.misc(let reason) {
            XCTAssertEqual(reason, decodeReason)
            XCTAssertEqual(attemptedUrls, [urls[0]])
        } catch {
            XCTFail("Expected the unchanged decode error, got \(error)")
        }
    }

    func testFirstMainnetResultDoesNotRetryCancellation() async throws {
        let urls = [
            URL(string: "http://192.0.2.1:9053")!,
            URL(string: "http://192.0.2.2:9053")!,
        ]
        var attemptedUrls: [URL] = []

        do {
            let _: Int = try await firstMainnetResult(urls: urls) { url, _, _ in
                attemptedUrls.append(url)
                throw CancellationError()
            }
            XCTFail("Expected cancellation to be rethrown")
        } catch is CancellationError {
            XCTAssertEqual(attemptedUrls, [urls[0]])
        } catch {
            XCTFail("Expected the unchanged cancellation, got \(error)")
        }
    }

    func testDistinctNodeUrlsUsesEffectiveHostAndPortIdentity() throws {
        let first = URL(string: "http://192.0.2.1:9053/first")!
        let sameNodeAlias = URL(string: "https://192.0.2.1:9053/second")!
        let sameHostDifferentPort = URL(string: "http://192.0.2.1:9054/third")!
        let second = URL(string: "http://192.0.2.2:9053/third")!

        XCTAssertEqual(
            try distinctNodeUrlsByEffectiveIdentity(
                [first, sameNodeAlias, sameHostDifferentPort, second]
            ),
            [first, sameHostDifferentPort, second]
        )
    }

    func testExternalNodeAvailabilityClassifierAcceptsRequestAndTimeoutReasons() {
        let url = URL(string: "http://192.0.2.1:9053")!
        let requestReason = "ReqwestError(reqwest::Error { kind: Request, "
            + "source: ConnectError(\"connection refused\") })"
        let timeoutReason = "ReqwestError(reqwest::Error { kind: Request, source: TimedOut })"
        let timeoutError = MainnetNodeFallbackError(
            attemptedUrl: url,
            underlyingError: RestNodeApiError.misc(timeoutReason),
            underlyingErrorDescription: timeoutReason
        )

        XCTAssertEqual(
            externalNodeAvailabilityReason(for: RestNodeApiError.misc(requestReason)),
            requestReason
        )
        XCTAssertEqual(externalNodeAvailabilityReason(for: timeoutError), timeoutReason)
    }

    func testExternalNodeAvailabilityClassifierRejectsDecodeAndLocalErrors() {
        let url = URL(string: "http://192.0.2.1:9053")!
        let decodeReason = "ReqwestError(reqwest::Error { kind: Decode, "
            + "source: Error(\"request timed out while decoding\") })"
        let decodeError = MainnetNodeFallbackError(
            attemptedUrl: url,
            underlyingError: RestNodeApiError.misc(decodeReason),
            underlyingErrorDescription: decodeReason
        )
        let localError = MainnetNodeFallbackError(
            attemptedUrl: url,
            underlyingError: SyntheticNodeError.unavailable(url),
            underlyingErrorDescription: "synthetic failure"
        )

        XCTAssertNil(externalNodeAvailabilityReason(for: decodeError))
        XCTAssertNil(externalNodeAvailabilityReason(for: localError))
        XCTAssertNil(
            externalNodeAvailabilityReason(for: RestNodeApiError.misc("synthetic timeout"))
        )
    }

    func testCallbackResultStateAllowsOnlyCompletionOrCloseToWin() throws {
        let completionWins = CallbackResultState<Int>()
        let completionExpectation = expectation(description: "completion wins")
        completionExpectation.assertForOverFulfill = true
        XCTAssertTrue(
            completionWins.complete(.success(42), expectation: completionExpectation)
        )
        XCTAssertFalse(
            completionWins.complete(.success(43), expectation: completionExpectation)
        )
        wait(for: [completionExpectation], timeout: 0.01)
        XCTAssertEqual(try completionWins.closeAndTakeResult()?.get(), 42)
        XCTAssertNil(completionWins.closeAndTakeResult())

        let closeWins = CallbackResultState<Int>()
        let lateExpectation = expectation(description: "late completion is suppressed")
        lateExpectation.isInverted = true
        XCTAssertNil(closeWins.closeAndTakeResult())
        XCTAssertFalse(closeWins.complete(.success(42), expectation: lateExpectation))
        wait(for: [lateExpectation], timeout: 0.01)
    }

    private func assertValidCallbackResult(
        _ result: Result<NipopowProof, Error>
    ) throws {
        switch result {
        case .success(let proof):
            XCTAssertNoThrow(try proof.toJSON()!)
        case .failure(let error):
            guard let reason = externalNodeAvailabilityReason(for: error) else {
                throw error
            }
            throw XCTSkip("Public mainnet node unavailable: \(reason)")
        }
    }

    func testGetNipopowProofByHeaderIdNonAsync() async throws {
        let expectation = self.expectation(description: "getNipopowByHeaderIdNonAsync")
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let nodeConf = try NodeConf(withUrl: mainnetNodeUrls[0])
        let restNodeApi = try RestNodeApi()
        let callbackState = CallbackResultState<NipopowProof>()
        let requestHandle = try restNodeApi.getNipopowProofByHeaderId(
            nodeConf: nodeConf,
            minChainLength: UInt32(3),
            suffixLen: UInt32(2),
            headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
        ) { result in
            callbackState.complete(result, expectation: expectation)
        }

        // The request itself has the upstream 30-second timeout; retain the handle through
        // callback delivery and leave a bounded scheduling margin for the main queue.
        let waiterResult = await XCTWaiter.fulfillment(of: [expectation], timeout: 45)
        let callbackResult = callbackState.closeAndTakeResult()
        switch waiterResult {
        case .completed:
            guard let result = callbackResult else {
                XCTFail("Callback completed without recording a result")
                return
            }
            try assertValidCallbackResult(result)
        default:
            if let result = callbackResult {
                try assertValidCallbackResult(result)
                return
            }
            requestHandle.abort()
            XCTFail("Callback did not complete before the waiter finished: \(waiterResult)")
            return
        }
    }

    func testGetNipopowProofByHeaderAbort() throws {
        let nodeConf = try NodeConf(withUrl: mainnetNodeUrls[0])
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
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let (proof, proofNew) = try await firstMainnetResult { _, restNodeApi, nodeConf in
            let proof = try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(3),
                suffixLen: UInt32(2),
                headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
            )

            // test re-use of the same Tokio runtime
            let proofNew = try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(3),
                suffixLen: UInt32(2),
                headerId: blockHeaders.get(index: UInt(0))!.getBlockId()
            )
            return (proof, proofNew)
        }

        XCTAssertNoThrow(try proof.toJSON()!)
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
        let headerId = try BlockId(
            withString: "d1366f762e46b7885496aaab0c42ec2950b0422d48aec3b91f45d4d0cdeb41e5")
        let txId = try TxId(
            withString: "258ddfc09b94b8313bca724de44a0d74010cab26de379be845713cc129546b78")
        // Get NiPoPow proofs from exactly 2 separate configured Ergo nodes.
        var proofs: [NipopowProof] = []
        var externalNodeReasons: [String] = []
        for url in try distinctNodeUrlsByEffectiveIdentity(mainnetNodeUrls) {
            if proofs.count >= 2 { break }
            do {
                if let proof = try await getNipopowProof(url: url, headerId: headerId) {
                    proofs.append(proof)
                } else {
                    externalNodeReasons.append(
                        "\(url.absoluteString): node does not expose the required NiPoPoW API"
                    )
                }
            } catch {
                guard let reason = externalNodeAvailabilityReason(for: error) else {
                    throw error
                }
                externalNodeReasons.append("\(url.absoluteString): \(reason)")
            }
        }
        guard proofs.count == 2 else {
            let diagnostics = externalNodeReasons.joined(separator: " | ")
            throw XCTSkip(
                "SPV workflow requires NiPoPoW proofs from two configured public nodes; "
                    + "received \(proofs.count). External-node diagnostics: \(diagnostics)"
            )
        }

        let genesisBlockId = try BlockId(
            withString: "b0244dfc267baca974a4caee06120321562784303a8a688976ae56170e4d175b")
        let verifier = NipopowVerifier(withGenesisBlockId: genesisBlockId)
        for proof in proofs {
            try verifier.process(newProof: proof)
        }
        let bestProof = verifier.bestProof()
        XCTAssertEqual(try bestProof.suffixHead().getHeader().getBlockId(), headerId)

        let (header, merkleProof) = try await firstMainnetResult {
            _, restNodeApi, nodeConf in
            let header = try await restNodeApi.getHeaderAsync(
                nodeConf: nodeConf,
                blockId: headerId
            )
            let merkleProof = try await restNodeApi.getBlocksHeaderIdProofForTxIdAsync(
                nodeConf: nodeConf,
                blockId: headerId,
                txId: txId
            )
            return (header, merkleProof)
        }
        XCTAssert(try merkleProof.valid(expected_root: header.getTransactionsRoot()))
    }
}

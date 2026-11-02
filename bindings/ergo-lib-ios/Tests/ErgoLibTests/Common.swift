import Foundation

@testable import ErgoLib
@testable import ErgoLibC

// The following two functions copied from: https://stackoverflow.com/a/49890939

func toPairsOfChars(pairs: [String], string: String) -> [String] {
    if string.count == 0 {
        return pairs
    }
    var pairsMod = pairs
    pairsMod.append(String(string.prefix(2)))
    return toPairsOfChars(pairs: pairsMod, string: String(string.dropFirst(2)))
}

func base16StringToBytes(_ string: String) -> [UInt8]? {
    // omit error checking: remove '0x', make sure even, valid chars
    let pairs = toPairsOfChars(pairs: [], string: string)
    return pairs.map { UInt8($0, radix: 16)! }
}

func getSeeds() -> [URL] {
    [
        "http://213.239.193.208:9030",
        "http://159.65.11.55:9030",
        "http://165.227.26.175:9030",
        "http://159.89.116.15:9030",
        "http://136.244.110.145:9030",
        "http://94.130.108.35:9030",
        "http://51.75.147.1:9020",
        "http://221.165.214.185:9030",
        "http://51.81.185.231:9031",
        "http://217.182.197.196:9030",
        "http://62.171.190.193:9030",
        "http://173.212.220.9:9030",
        "http://176.9.65.58:9130",
        "http://213.152.106.56:9030",
    ].map { URL(string: $0)! }
}

func getNipopowProof(url: URL, headerId: BlockId) async throws -> NipopowProof? {
    try await firstMainnetResult(urls: [url]) { _, restNodeApi, nodeConf in
        let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf)
        if nodeInfo.isAtLeastVersion40100() {
            return try await restNodeApi.getNipopowProofByHeaderIdAsync(
                nodeConf: nodeConf,
                minChainLength: UInt32(7),
                suffixLen: UInt32(6),
                headerId: headerId
            )
        }
        return nil
    }
}

// Known-live mainnet nodes (REST API on :9053) used as fallbacks for network-dependent tests.
let mainnetNodeUrls = [
    "http://213.239.193.208:9053",
    "http://159.65.11.55:9053",
].map { URL(string: $0)! }

struct MainnetNodeFallbackError: Error, LocalizedError {
    let attemptedUrl: URL?
    let underlyingError: Error?
    let underlyingErrorDescription: String

    var errorDescription: String? {
        guard let attemptedUrl = attemptedUrl else {
            return underlyingErrorDescription
        }
        return "Mainnet node operation failed at \(attemptedUrl.absoluteString): "
            + underlyingErrorDescription
    }
}

func externalNodeAvailabilityReason(for error: Error) -> String? {
    let underlyingError: Error
    if let fallbackError = error as? MainnetNodeFallbackError {
        guard let fallbackUnderlyingError = fallbackError.underlyingError else {
            return nil
        }
        underlyingError = fallbackUnderlyingError
    } else {
        underlyingError = error
    }

    guard let restError = underlyingError as? RestNodeApiError else {
        return nil
    }
    let reason: String
    switch restError {
    case .misc(let restReason):
        reason = restReason
    }

    guard !reason.contains("kind: Decode") else {
        return nil
    }
    guard reason.contains("ReqwestError") else {
        return nil
    }

    let externalMarkers = [
        "kind: Request",
        "ConnectError",
        "ConnectionRefused",
        "TimedOut",
        "timed out",
        "timeout",
    ]
    guard externalMarkers.contains(where: { reason.contains($0) }) else {
        return nil
    }
    return reason
}

func effectiveNodeIdentity(_ url: URL) throws -> String {
    guard url.scheme == "http" || url.scheme == "https" else {
        throw NodeConfError.InvalidScheme
    }
    guard let host = url.host else { throw NodeConfError.MissingHost }
    guard let port = url.port else { throw NodeConfError.MissingPort }
    return host + ":" + String(port)
}

func distinctNodeUrlsByEffectiveIdentity(_ urls: [URL]) throws -> [URL] {
    var identities = Set<String>()
    var distinctUrls: [URL] = []
    for url in urls {
        let identity = try effectiveNodeIdentity(url)
        if identities.insert(identity).inserted {
            distinctUrls.append(url)
        }
    }
    return distinctUrls
}

private func nodeOperationErrorDescription(_ error: Error) -> String {
    if let restError = error as? RestNodeApiError {
        switch restError {
        case .misc(let reason):
            return reason
        }
    }
    return error.localizedDescription
}

func firstMainnetResult<T>(
    urls: [URL] = mainnetNodeUrls,
    operation: (URL, RestNodeApi, NodeConf) async throws -> T
) async throws -> T {
    var lastError: MainnetNodeFallbackError?
    for url in urls {
        do {
            let nodeConf = try NodeConf(withUrl: url)
            let restNodeApi = try RestNodeApi()
            return try await operation(url, restNodeApi, nodeConf)
        } catch {
            guard externalNodeAvailabilityReason(for: error) != nil else {
                throw error
            }
            lastError = MainnetNodeFallbackError(
                attemptedUrl: url,
                underlyingError: error,
                underlyingErrorDescription: nodeOperationErrorDescription(error)
            )
        }
    }

    if let lastError = lastError {
        throw lastError
    }
    throw MainnetNodeFallbackError(
        attemptedUrl: nil,
        underlyingError: nil,
        underlyingErrorDescription: "No mainnet node URLs were configured"
    )
}

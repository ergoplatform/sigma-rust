import Foundation
import ErgoLibC
import Dispatch

enum RestNodeApiError: Error {
    case misc(String)
}

class RestNodeApi {
    internal var pointer: RestApiRuntimePtr

    /// Create ergo ``RestNodeApi`` instance 
    init() throws {
        var ptr: RestApiRuntimePtr?
        let error = ergo_lib_rest_api_runtime_create(&ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// GET on /info endpoint
    func getInfo(
        nodeConf: NodeConf,
        closure: @escaping (Result<NodeInfo, Error>) -> Void
    ) throws -> RequestHandle {
        
        let completion = wrapClosure(closure)
        var requestHandlerPtr: RequestHandlePtr?
        let error = ergo_lib_rest_api_node_get_info(self.pointer, nodeConf.pointer, 
            completion, &requestHandlerPtr)
        try checkError(error)
        return RequestHandle(withRawPtr: requestHandlerPtr!)
    }
    
    /// GET on /info endpoint (async)
    @available(macOS 10.15, iOS 13, *)
    func getInfoAsync(nodeConf: NodeConf) async throws -> NodeInfo {
        try await withCheckedThrowingContinuation { continuation in
            do {
                let _ = try getInfo(nodeConf: nodeConf) { result in
                    continuation.resume(with: result)
                }
            } catch {}
        }
    }

    /// GET on /blocks/{blockId}/header endpoint
    func getHeader(
        nodeConf: NodeConf,
        blockId: BlockId,
        closure: @escaping (Result<BlockHeader, Error>) -> Void
    ) throws -> RequestHandle {
        let completion = wrapClosure(closure)
        var requestHandlerPtr: RequestHandlePtr?
        let error = ergo_lib_rest_api_node_get_header(
            self.pointer,
            nodeConf.pointer,
            completion,
            &requestHandlerPtr,
            blockId.pointer
        )
        try checkError(error)
        return RequestHandle(withRawPtr: requestHandlerPtr!)
    }

    /// GET on /blocks/{blockId}/header endpoint (async)
    @available(macOS 10.15, iOS 13, *)
    func getHeaderAsync(nodeConf: NodeConf, blockId: BlockId) async throws -> NodeInfo {
        try await withCheckedThrowingContinuation { continuation in
            do {
                let _ = try getInfo(nodeConf: nodeConf, blockId: blockId) { result in
                    continuation.resume(with: result)
                }
            } catch {}
        }
    }

    /// GET on /blocks/{header_id}/proofFor/{tx_id} to request the merkle proof for a given transaction
    /// that belongs to the given header ID.
    func getBlocksHeaderIdProofForTxId(
        nodeConf: NodeConf,
        blockId: BlockId,
        txId: TxId,
        closure: @escaping (Result<MerkleProof, Error>) -> Void
    ) throws -> RequestHandle {
        let completion = wrapClosure(closure)
        var requestHandlerPtr: RequestHandlePtr?
        let error = ergo_lib_rest_api_node_get_blocks_header_id_proof_for_tx_id(
            self.pointer,
            nodeConf.pointer,
            completion,
            &requestHandlerPtr,
            blockId.pointer,
            txId.pointer
        )
        try checkError(error)
        return RequestHandle(withRawPtr: requestHandlerPtr!)
    }

    /// GET on /blocks/{header_id}/proofFor/{tx_id} to request the merkle proof for a given transaction
    /// that belongs to the given header ID (async).
    @available(macOS 10.15, iOS 13, *)
    func getBlocksHeaderIdProofForTxIdAsync(nodeConf: NodeConf, blockId: BlockId, txId: TxId) async throws -> NodeInfo {
        try await withCheckedThrowingContinuation { continuation in
            do {
                let _ = try getBlocksHeaderIdProofForTxId(nodeConf: nodeConf, blockId: blockId, txId: txId) { result in
                    continuation.resume(with: result)
                }
            } catch {}
        }
    }

    /// GET on /nipopow/proof/{minChainLength}/{suffixLength}/{headerId} endpoint
    func getNipopowProofByHeaderId(
        nodeConf: NodeConf,
        minChainLength: UInt32,
        suffixLen: UInt32,
        headerId: BlockId,
        closure: @escaping (Result<NipopowProof, Error>) -> Void
    ) throws -> RequestHandle {
        
        let completion = wrapClosure(closure)
        var requestHandlerPtr: RequestHandlePtr?
        let error = ergo_lib_rest_api_node_get_nipopow_proof_by_header_id(
            self.pointer,
            nodeConf.pointer,
            completion,
            &requestHandlerPtr,
            minChainLength,
            suffixLen,
            headerId.pointer
        )
        try checkError(error)
        return RequestHandle(withRawPtr: requestHandlerPtr!)
    }
    
    /// GET on /info endpoint (async)
    @available(macOS 10.15, iOS 13, *)
    func getNipopowProofByHeaderIdAsync(
        nodeConf: NodeConf,
        minChainLength: UInt32,
        suffixLen: UInt32,
        headerId: BlockId
    ) async throws -> NipopowProof {
        try await withCheckedThrowingContinuation { continuation in
            do {
                let _ = try getNipopowProofByHeaderId(
                    nodeConf: nodeConf,
                    minChainLength: minChainLength,
                    suffixLen: suffixLen,
                    headerId: headerId
                ) { result in
                    continuation.resume(with: result)
                }
            } catch {
            }
        }
    }

    /// Given a list of seed nodes, search for peer nodes with an active REST API on port 9053.
    ///  - `seeds` represents a list of ergo node URLs from which to start peer discovery.
    ///  - `max_parallel_requests` represents the maximum number of HTTP requests that can be made in
    ///    parallel
    ///  - `timeout` represents the amount of time that is spent search for peers. Once the timeout
    ///    value is reached, return with the vec of active peers that have been discovered up to that
    ///    point in time.
    func peerDiscovery(
        seeds: [URL],
        maxParallelReqs: UInt16,
        timeoutSec: UInt32,
        closure: @escaping (Result<CStringCollection, Error>) -> Void
    ) throws -> RequestHandle {
        let completion = wrapClosure(closure)
        var requestHandlerPtr: RequestHandlePtr?
        // Need to convert `seeds` into [String] then clone it in a C-compatible way. This allows
        // us to send it across the FFI boundary.
        var ptr = seeds.map{ UnsafePointer<CChar>(strdup($0.absoluteString)) }
        let error = ergo_lib_rest_api_node_peer_discovery(
            self.pointer,
            completion,
            &requestHandlerPtr,
            &ptr,
            UInt(seeds.count),
            maxParallelReqs,
            timeoutSec
        )
        
        // Deallocate the memory pointed-to by `ptr`
        for p in ptr { free(UnsafeMutablePointer(mutating: p)) }
        
        try checkError(error)
        return RequestHandle(withRawPtr: requestHandlerPtr!)
    }
    
    
    /// Given a list of seed nodes, search for peer nodes with an active REST API on port 9053.
    ///  - `seeds` represents a list of ergo node URLs from which to start peer discovery.
    ///  - `max_parallel_requests` represents the maximum number of HTTP requests that can be made in
    ///    parallel
    ///  - `timeout` represents the amount of time that is spent search for peers. Once the timeout
    ///    value is reached, return with the vec of active peers that have been discovered up to that
    ///    point in time.
    @available(macOS 10.15, iOS 13, *)
    func peerDiscoveryAsync(
        seeds: [URL],
        maxParallelReqs: UInt16,
        timeoutSec: UInt32
    ) async throws -> [URL] {
        try await withCheckedThrowingContinuation { continuation in
            do {
                let _ = try peerDiscovery(
                    seeds: seeds, maxParallelReqs: maxParallelReqs, timeoutSec: timeoutSec
                ) { (result: Result<CStringCollection, Error>) in
                    let r = result.map { $0.toArray().compactMap {URL(string: $0)}}
                    continuation.resume(with: r)
                }
            } catch {}
        }
    }

    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

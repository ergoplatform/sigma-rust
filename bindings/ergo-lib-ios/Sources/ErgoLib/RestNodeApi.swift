import Foundation
import ErgoLibC
import Dispatch

class RestNodeApi {
    internal var pointer: RestApiRuntimePtr

    /// Create ergo ``RestNodeApi`` instance 
    init() throws {
        var ptr: RestApiRuntimePtr?
        let error = ergo_lib_rest_api_runtime_create(&ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    func getInfo(nodeConf: NodeConf, timeoutSec: UInt32) throws -> NodeInfo {
        var ptr: NodeInfoPtr?
            let error = ergo_lib_rest_api_node_get_info(self.pointer, nodeConf.pointer, timeoutSec, &ptr)
            try checkError(error)
            return NodeInfo(withRawPointer: ptr!)
    }

    /// Async wrapper with a callback running in the background queue
    /* private func getInfoCallback( */
    /*     nodeConf: NodeConf, */
    /*     callback: @escaping (Result<NodeInfo, Error>) -> Void) { */

    /*     DispatchQueue.global(qos: .background).async { */
    /*         let res = Result { try self.getInfo(nodeConf: nodeConf) } */
    /*         callback(res) */
    /*     } */
    /* } */

    // need macOS 12.0 on GA
    /* func getInfoAsync(nodeConf: NodeConf) async throws -> NodeInfo { */
    /*     try await withCheckedThrowingContinuation { continuation in */
    /*         getInfoCallback(nodeConf: nodeConf) { result in */ 
    /*             continuation.resume(with: result) */
    /*         } */
    /*     } */
    /* } */

    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

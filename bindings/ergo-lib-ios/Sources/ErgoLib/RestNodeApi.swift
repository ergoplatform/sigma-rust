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


    // need macOS 12.0 on GA
    func getInfoAsync(nodeConf: NodeConf) async throws -> NodeInfo {
        try await withCheckedThrowingContinuation { continuation in
            getInfo(nodeConf: nodeConf) { result in 
                continuation.resume(with: result)
            }
        }
    }

    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

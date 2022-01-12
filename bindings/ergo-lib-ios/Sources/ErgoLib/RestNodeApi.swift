import Foundation
import ErgoLibC

class RestNodeApi {
    internal var pointer: RestApiRuntimePtr

    /// Create ergo ``RestNodeApi`` instance 
    init() throws {
        var ptr: RestApiRuntimePtr?
        let error = ergo_lib_rest_api_runtime_create(&ptr)
        try checkError(error)
        self.pointer = ptr!
    }

    func getInfo(nodeConf: NodeConf) throws -> NodeInfo {
        var ptr: NodeInfoPtr?
            let error = ergo_lib_rest_api_node_get_info(self.pointer, nodeConf.pointer, &ptr)
            try checkError(error)
            return NodeInfo(withRawPointer: ptr!)

    }

    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

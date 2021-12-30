import Foundation
import ErgoLibC

class RestNodeApi {

    static func getInfo(nodeConf: NodeConf) throws -> NodeInfo {
        var ptr: NodeInfoPtr?
            let error = ergo_lib_rest_api_node_get_info(nodeConf.pointer, &ptr)
            try checkError(error)
            return NodeInfo(withRawPointer: ptr!)

    }
}

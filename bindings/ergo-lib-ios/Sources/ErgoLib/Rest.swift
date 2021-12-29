import Foundation
import ErgoLibC

class RestNode {
    internal var pointer: RestApiRuntimePtr
    
    /// Create ergo node ``Rest`` API instance 
    init() throws {
        var ptr: RestApiRuntimePtr?
        let error = ergo_lib_rest_api_runtime_create(&ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// GET on /info endpoint
    func get_info(
        nodeConf: NodeConf,
        closureSuccess: @escaping (NodeInfo) -> Void,
        closureFail: @escaping (String) -> Void
    ) throws {
        // step 1
        let wrappedClosureSuccess = WrapClosure(closure: closureSuccess)
        let userdataSuccess = Unmanaged.passRetained(wrappedClosureSuccess).toOpaque()
        // step 1
        let wrappedClosureFail = WrapClosure(closure: closureFail)
        let userdataFail = Unmanaged.passRetained(wrappedClosureFail).toOpaque()

        // step 2
        let callback_success: @convention(c) (UnsafeMutableRawPointer, NodeInfoPtr) -> Void = { (_ userdata: UnsafeMutableRawPointer, _ nodeInfoPtr: NodeInfoPtr) in
            let wrappedClosure: WrapClosure<(NodeInfo) -> Void> = Unmanaged.fromOpaque(userdata).takeRetainedValue()
                    let nodeInfo = NodeInfo(withRawPointer: nodeInfoPtr)
                    wrappedClosure.closure(nodeInfo)
        }

        // step 2
        let callback_fail: @convention(c) (UnsafeMutableRawPointer, ErrorPtr) -> Void = { (_ userdata: UnsafeMutableRawPointer, _ errorPtr: ErrorPtr) in
            let wrappedClosure: WrapClosure<(String) -> Void> = Unmanaged.fromOpaque(userdata).takeRetainedValue()
                    let cStringReason = ergo_lib_error_to_string(errorPtr)
                    let reason = String(cString: cStringReason!)
                    ergo_lib_delete_string(cStringReason)
                    ergo_lib_delete_error(errorPtr)
                    wrappedClosure.closure(reason)
        }

        // step 3
        let completion = CompletedCallback_NodeInfo(userdata_success: userdataSuccess, 
            userdata_fail: userdataFail, callback_success: callback_success, callback_fail: callback_fail)

        let error = ergo_lib_rest_api_node_get_info_async(self.pointer, nodeConf.pointer, completion)

        try checkError(error)
    }
    
    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

private class WrapClosure<T> {
    fileprivate let closure: T
    init(closure: T) {
        self.closure = closure
    }
}

class NodeConf {
    internal var pointer: NodeConfPtr

    internal init(withRawPointer ptr: NodeConfPtr) {
        self.pointer = ptr
    }

    deinit {
        ergo_lib_node_conf_delete(self.pointer)
    }
}

class NodeInfo {
    internal var pointer: NodeInfoPtr

    internal init(withRawPointer ptr: NodeInfoPtr) {
        self.pointer = ptr
    }

    deinit {
        ergo_lib_node_info_delete(self.pointer)
    }
}

func getInfo(nodeConf: NodeConf) throws -> NodeInfo {
    var ptr: NodeInfoPtr?
    let error = ergo_lib_rest_api_node_get_info(nodeConf.pointer, &ptr)
    try checkError(error)
    return NodeInfo(withRawPointer: ptr!)
}

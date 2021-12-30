import Foundation
import ErgoLibC


private class WrapClosure<T> {
    fileprivate let closure: T
    init(closure: T) {
        self.closure = closure
    }
}

class RestNodeApiAsync {
    internal var pointer: RestApiRuntimePtr
    
    /// Create ergo ``RestNodeApi`` instance 
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
        // base on https://www.nickwilcox.com/blog/recipe_swift_rust_callback/
        // step 1: manually increment reference count on both closures
        let wrappedClosureSuccess = WrapClosure(closure: closureSuccess)
        let userdataSuccess = Unmanaged.passRetained(wrappedClosureSuccess).toOpaque()
        let wrappedClosureFail = WrapClosure(closure: closureFail)
        let userdataFail = Unmanaged.passRetained(wrappedClosureFail).toOpaque()

        // step 2: create C compatible function pointer
        let callback_success: @convention(c) (UnsafeMutableRawPointer, NodeInfoPtr) -> Void = { (_ userdata: UnsafeMutableRawPointer, _ nodeInfoPtr: NodeInfoPtr) in
            // reverse step 1 and manually decrement reference count on the closure and turn it back to Swift type.
            // Because we are back to letting Swift manage our reference count, when the scope ends the wrapped closure will be freed.
            let wrappedClosure: WrapClosure<(NodeInfo) -> Void> = Unmanaged.fromOpaque(userdata).takeRetainedValue()
                    let nodeInfo = NodeInfo(withRawPointer: nodeInfoPtr)
                    // TODO: call it on the same thread  `get_info` was called (i.e. on UI thread)
                    wrappedClosure.closure(nodeInfo)
        }
        let callback_fail: @convention(c) (UnsafeMutableRawPointer, ErrorPtr) -> Void = { (_ userdata: UnsafeMutableRawPointer, _ errorPtr: ErrorPtr) in
            let wrappedClosure: WrapClosure<(String) -> Void> = Unmanaged.fromOpaque(userdata).takeRetainedValue()
                    let cStringReason = ergo_lib_error_to_string(errorPtr)
                    let reason = String(cString: cStringReason!)
                    ergo_lib_delete_string(cStringReason)
                    ergo_lib_delete_error(errorPtr)
                    // TODO: call it on the same thread  `get_info` was called (i.e. on UI thread)
                    wrappedClosure.closure(reason)
        }

        let completion = CompletedCallback_NodeInfo(userdata_success: userdataSuccess, 
            userdata_fail: userdataFail, callback_success: callback_success, callback_fail: callback_fail)

        let error = ergo_lib_rest_api_node_get_info_async(self.pointer, nodeConf.pointer, completion)

        try checkError(error)
    }
    
    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

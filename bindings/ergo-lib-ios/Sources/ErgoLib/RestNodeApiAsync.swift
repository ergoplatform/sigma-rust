import Foundation
import ErgoLibC


private class WrapClosure<T> {
    fileprivate let closure: T
    init(closure: T) {
        self.closure = closure
    }
}

enum RestNodeApiError: Error {
    case misc(String)
}

class RestNodeApiAsync {
    internal var pointer: RestApiRuntimePtr
    
    /// Create ergo ``RestNodeApiAsync`` instance 
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
        let error = ergo_lib_rest_api_node_get_info_async(self.pointer, nodeConf.pointer, 
            completion, &requestHandlerPtr)
        try checkError(error)
        return RequestHandle(withRawPtr: requestHandlerPtr!)
    }
    
    deinit {
        ergo_lib_rest_api_runtime_delete(self.pointer)
    }
}

func wrapClosure<T: FromRawPtr>(_ closure: @escaping (Result<T, Error>) -> Void) -> CompletedCallback {
        let closure: (Result<UnsafeRawPointer, Error>) -> Void = {
            (res: Result<UnsafeRawPointer, Error>) in
            let mapped = res.map { rawPtr in 
                T.fromRawPtr(ptr: rawPtr)
            }
            closure(mapped)
        }
        return wrapClosureRawPtr(closure)
}

func wrapClosureRawPtr(_ closure: @escaping (Result<UnsafeRawPointer, Error>) -> Void) -> CompletedCallback {
    // base on https://www.nickwilcox.com/blog/recipe_swift_rust_callback/
    // step 1: manually increment reference count on closure
    let wrappedClosure = WrapClosure(closure: closure)
    let userdata = Unmanaged.passRetained(wrappedClosure).toOpaque()

    // step 2: create C compatible function pointer
    let callback: @convention(c) (UnsafeMutableRawPointer, UnsafeRawPointer?, ErrorPtr?) -> Void = {
        (_ userdata: UnsafeMutableRawPointer, _ resPtr: UnsafeRawPointer?,  _ errorPtr: ErrorPtr?) in
        // reverse step 1 and manually decrement reference count on the closure and turn it back to Swift type.
        // Because we are back to letting Swift manage our reference count, when the scope ends the wrapped closure will be freed.
        let wrappedClosure: WrapClosure<(Result<UnsafeRawPointer, Error>) -> Void> =
            Unmanaged.fromOpaque(userdata).takeRetainedValue()

        if let resPtr = resPtr {
            // success
            let res = Result<UnsafeRawPointer, Error>.success(resPtr)
            // TODO: call it on the same thread  `get_info` was called (on main/UI thread?)
            wrappedClosure.closure(res)
        } else {
            // failure
            let cStringReason = ergo_lib_error_to_string(errorPtr!)
            let reason = String(cString: cStringReason!)
            ergo_lib_delete_string(cStringReason)
            ergo_lib_delete_error(errorPtr)
            let res = Result<UnsafeRawPointer, Error>.failure(RestNodeApiError.misc(reason))
            // TODO: call it on the same thread  `get_info` was called (on main/UI thread?)
            wrappedClosure.closure(res)
        }
    }

    let callback_release: @convention(c) (UnsafeMutableRawPointer) -> Void = { (_ userdata: UnsafeMutableRawPointer) in
        // reverse step 1 and manually decrement reference count on the closure and turn it back to Swift type.
        // Because we are back to letting Swift manage our reference count, when the scope ends 
        // the wrapped closure will be freed.
        let _ :WrapClosure<(Result<UnsafeRawPointer, Error>) -> Void> = 
            Unmanaged.fromOpaque(userdata).takeRetainedValue()
    }

    return CompletedCallback( userdata: userdata, callback: callback, callback_release: callback_release)
}

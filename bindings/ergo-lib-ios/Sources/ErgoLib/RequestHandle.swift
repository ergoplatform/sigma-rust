import Foundation
import ErgoLibC

/// Rest API request handle
class RequestHandle {
    internal var pointer: RequestHandlePtr
    
    internal init(withRawPtr ptr: ContractPtr) {
        self.pointer = ptr
    }
    
    /// Abort the request
    func abort() {
        ergo_lib_rest_api_request_handle_abort(self.pointer)
    }
        
    deinit {
        ergo_lib_rest_api_request_handle_delete(self.pointer)
    }
}

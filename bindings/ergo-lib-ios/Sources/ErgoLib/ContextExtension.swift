
import Foundation
import ErgoLibC
import SwiftyJSON

class ContextExtension {
    internal var pointer: ContextExtensionPtr
    
    init() throws {
        self.pointer = try ContextExtension.fromEmpty()
    }
    
    init(withPtr ptr: ContextExtensionPtr) {
        self.pointer = ptr
    }
    
    private static func fromEmpty() throws -> ContextExtensionPtr {
        var ptr: ContextExtensionPtr?
        let error = ergo_wallet_context_extension_empty(&ptr)
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_context_extension_delete(self.pointer)
    }
}

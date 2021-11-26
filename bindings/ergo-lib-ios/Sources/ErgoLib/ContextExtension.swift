
import Foundation
import ErgoLibC
import SwiftyJSON

class ContextExtension {
    internal var pointer: ContextExtensionPtr
    
    init() {
        var ptr: ContextExtensionPtr?
        ergo_wallet_context_extension_empty(&ptr)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: ContextExtensionPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_wallet_context_extension_delete(self.pointer)
    }
}

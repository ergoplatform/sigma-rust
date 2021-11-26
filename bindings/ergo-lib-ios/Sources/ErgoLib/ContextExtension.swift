
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
    
    func getKeys() -> [UInt8] {
        let bytesLength = ergo_wallet_context_extension_len(self.pointer)
        var bytes = Array.init(repeating: UInt8(0), count: Int(bytesLength))
        ergo_wallet_context_extension_keys(self.pointer, &bytes)
        return bytes
    }
    
    internal init(withPtr ptr: ContextExtensionPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_wallet_context_extension_delete(self.pointer)
    }
}

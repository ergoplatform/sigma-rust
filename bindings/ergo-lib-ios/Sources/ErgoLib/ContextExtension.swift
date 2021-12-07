
import Foundation
import ErgoLibC

/// User-defined variables to be put into context
class ContextExtension {
    internal var pointer: ContextExtensionPtr
    
    /// Returns an empty ``ContextExtension``
    init() {
        var ptr: ContextExtensionPtr?
        ergo_lib_context_extension_empty(&ptr)
        self.pointer = ptr!
    }
    
    /// Returns all keys (``UInt8`` values) in the map
    func getKeys() -> [UInt8] {
        let bytesLength = ergo_lib_context_extension_len(self.pointer)
        var bytes = Array.init(repeating: UInt8(0), count: Int(bytesLength))
        ergo_lib_context_extension_keys(self.pointer, &bytes)
        return bytes
    }
    
    /// Takes ownership of an existing ``ContextExtensionPtr``. Note: we must ensure that no other instance
    /// of ``ContextExtension`` can hold this pointer.
    internal init(withPtr ptr: ContextExtensionPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_lib_context_extension_delete(self.pointer)
    }
}

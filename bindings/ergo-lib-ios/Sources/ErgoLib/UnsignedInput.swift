import Foundation
import ErgoLibC

class UnsignedInput {
    internal var pointer: UnsignedInputPtr
    
    internal init(withPtr ptr: UnsignedInputPtr) {
        self.pointer = ptr
    }
    
    func getBoxId() throws -> BoxId {
        var boxIdPtr: BoxIdPtr?
        let error = ergo_wallet_unsigned_input_box_id(self.pointer, &boxIdPtr)
        try checkError(error)
        return BoxId(withPtr: boxIdPtr!)
    }
        
    func getContextExtension() throws -> ContextExtension {
        var contextExtensionPtr: ContextExtensionPtr?
        let error = ergo_wallet_unsigned_input_context_extension(self.pointer, &contextExtensionPtr)
        try checkError(error)
        return ContextExtension(withPtr: contextExtensionPtr!)
    }
    
    deinit {
        ergo_wallet_unsigned_input_delete(self.pointer)
    }
}

class UnsignedInputs {
    internal var pointer: UnsignedInputsPtr
    
    init() throws {
        self.pointer = try UnsignedInputs.initEmpty()
    }
    
    init(withPtr ptr: UnsignedInputsPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() throws -> UnsignedInputsPtr {
        var unsignedInputsPtr: UnsignedInputsPtr?
        let error = ergo_wallet_unsigned_inputs_new(&unsignedInputsPtr)
        try checkError(error)
        return unsignedInputsPtr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_unsigned_inputs_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> UnsignedInput? {
        var unsignedInputPtr: UnsignedInputPtr?
        let res = ergo_wallet_unsigned_inputs_get(self.pointer, index, &unsignedInputPtr)
        try checkError(res.error)
        if res.is_some {
            return UnsignedInput(withPtr: unsignedInputPtr!)
        } else {
            return nil
        }
    }
    
    func add(unsignedInput: UnsignedInput) throws {
        let error = ergo_wallet_unsigned_inputs_add(unsignedInput.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_unsigned_inputs_delete(self.pointer)
    }
}

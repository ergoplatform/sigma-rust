import Foundation
import ErgoLibC
import SwiftyJSON

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

class Input {
    internal var pointer: InputPtr
    
    internal init(withPtr ptr: InputPtr) {
        self.pointer = ptr
    }
    
    func getBoxId() throws -> BoxId {
        var boxIdPtr: BoxIdPtr?
        let error = ergo_wallet_input_box_id(self.pointer, &boxIdPtr)
        try checkError(error)
        return BoxId(withPtr: boxIdPtr!)
    }
        
    func getSpendingProof() throws -> ProverResult {
        var ptr: ProverResultPtr?
        let error = ergo_wallet_input_spending_proof(self.pointer, &ptr)
        try checkError(error)
        return ProverResult(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_wallet_input_delete(self.pointer)
    }
}

class ProverResult {
    internal var pointer: ProverResultPtr
    
    internal init(withRawPointer ptr: ProverResultPtr) {
        self.pointer = ptr
    }
    
    func toBytes() throws -> [UInt8] {
        let res = ergo_wallet_prover_result_proof_len(self.pointer)
        try checkError(res.error)
        var bytes = Array.init(repeating: UInt8(0), count: Int(res.value))
        let error = ergo_wallet_prover_result_proof(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    func getContextExtension() throws -> ContextExtension {
        var ptr: ContextExtensionPtr?
        let error = ergo_wallet_prover_result_context_extension(self.pointer, &ptr)
        try checkError(error)
        return ContextExtension(withPtr: ptr!)
    }
    
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_prover_result_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_wallet_prover_result_delete(self.pointer)
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

class Inputs {
    internal var pointer: InputsPtr
    
    init() throws {
        self.pointer = try Inputs.initEmpty()
    }
    
    init(withPtr ptr: InputsPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() throws -> InputsPtr {
        var ptr: InputsPtr?
        let error = ergo_wallet_inputs_new(&ptr)
        try checkError(error)
        return ptr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_inputs_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> Input? {
        var ptr: InputPtr?
        let res = ergo_wallet_inputs_get(self.pointer, index, &ptr)
        try checkError(res.error)
        if res.is_some {
            return Input(withPtr: ptr!)
        } else {
            return nil
        }
    }
    
    func add(input: Input) throws {
        let error = ergo_wallet_inputs_add(input.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_inputs_delete(self.pointer)
    }
}

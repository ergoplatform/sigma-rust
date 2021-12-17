import Foundation
import ErgoLibC

/// Unsigned inputs used in constructing unsigned transactions
class UnsignedInput {
    internal var pointer: UnsignedInputPtr
    
    /// Takes ownership of an existing ``UnsignedInputPtr``. Note: we must ensure that no other instance
    /// of ``UnsignedInput`` can hold this pointer.
    internal init(withRawPointer ptr: UnsignedInputPtr) {
        self.pointer = ptr
    }
    
    /// Get box id
    func getBoxId() -> BoxId {
        var boxIdPtr: BoxIdPtr?
        ergo_lib_unsigned_input_box_id(self.pointer, &boxIdPtr)
        return BoxId(withRawPointer: boxIdPtr!)
    }
        
    /// Get context extension
    func getContextExtension() -> ContextExtension {
        var contextExtensionPtr: ContextExtensionPtr?
        ergo_lib_unsigned_input_context_extension(self.pointer, &contextExtensionPtr)
        return ContextExtension(withPtr: contextExtensionPtr!)
    }
    
    deinit {
        ergo_lib_unsigned_input_delete(self.pointer)
    }
}

/// Signed inputs used in signed transactions
class Input {
    internal var pointer: InputPtr
    
    internal init(withRawPointer ptr: InputPtr) {
        self.pointer = ptr
    }
    
    /// Get box id
    func getBoxId() -> BoxId {
        var boxIdPtr: BoxIdPtr?
        ergo_lib_input_box_id(self.pointer, &boxIdPtr)
        return BoxId(withRawPointer: boxIdPtr!)
    }
        
    /// Get spending proof
    func getSpendingProof() -> ProverResult {
        var ptr: ProverResultPtr?
        ergo_lib_input_spending_proof(self.pointer, &ptr)
        return ProverResult(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_lib_input_delete(self.pointer)
    }
}

/// Proof of correctness of tx spending
class ProverResult {
    internal var pointer: ProverResultPtr
    
    internal init(withRawPointer ptr: ProverResultPtr) {
        self.pointer = ptr
    }
    
    /// Get proof bytes
    func toBytes() -> [UInt8] {
        let proofLength = ergo_lib_prover_result_proof_len(self.pointer)
        var bytes = Array.init(repeating: UInt8(0), count: Int(proofLength))
        ergo_lib_prover_result_proof(self.pointer, &bytes)
        return bytes
    }
    
    /// Get context extension
    func getContextExtension() -> ContextExtension {
        var ptr: ContextExtensionPtr?
        ergo_lib_prover_result_context_extension(self.pointer, &ptr)
        return ContextExtension(withPtr: ptr!)
    }
    
    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_prover_result_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_lib_prover_result_delete(self.pointer)
    }
}

/// An ordered collection of ``UnsignedInput``s
class UnsignedInputs {
    internal var pointer: UnsignedInputsPtr
    
    /// Create an empty collection
    init() {
        var ptr: UnsignedInputsPtr?
        ergo_lib_unsigned_inputs_new(&ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``UnsignedInputsPtr``. Note: we must ensure that no other instance
    /// of ``UnsignedInputs`` can hold this pointer.
    internal init(withRawPointer ptr: UnsignedInputsPtr) {
        self.pointer = ptr
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_lib_unsigned_inputs_len(self.pointer)
    }
    
    /// Returns the ``UnsignedInput`` at location `index` if it exists.
    func get(index: UInt) -> UnsignedInput? {
        var ptr: UnsignedInputPtr?
        let res = ergo_lib_unsigned_inputs_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return UnsignedInput(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    /// Add an ``UnsignedInput`` to the end of the collection.
    func add(unsignedInput: UnsignedInput) {
        ergo_lib_unsigned_inputs_add(unsignedInput.pointer, self.pointer)
    }
        
    deinit {
        ergo_lib_unsigned_inputs_delete(self.pointer)
    }
}

/// An ordered collection of ``Input``s
class Inputs {
    internal var pointer: InputsPtr
    
    /// Create an empty collection
    init() {
        var ptr: InputsPtr?
        ergo_lib_inputs_new(&ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``InputsPtr``. Note: we must ensure that no other instance
    /// of ``Inputs`` can hold this pointer.
    init(withRawPointer ptr: InputsPtr) {
        self.pointer = ptr
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_lib_inputs_len(self.pointer)
    }
    
    /// Returns the ``Input`` at location `index` if it exists.
    func get(index: UInt) -> Input? {
        var ptr: InputPtr?
        let res = ergo_lib_inputs_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return Input(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    /// Add an ``Input`` to the end of the collection.
    func add(input: Input) {
        ergo_lib_inputs_add(input.pointer, self.pointer)
    }
        
    deinit {
        ergo_lib_inputs_delete(self.pointer)
    }
}

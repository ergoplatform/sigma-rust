import Foundation
import ErgoLibC

/// Inputs, that are used to enrich script context, but won't be spent by the transaction
class DataInput {
    internal var pointer: DataInputPtr
    
    /// Parse box id (32 byte digest)
    init(withBoxId: BoxId) {
        var ptr: DataInputPtr?
        ergo_wallet_data_input_new(withBoxId.pointer, &ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``DataInputPtr``. Note: we must ensure that no other instance
    /// of ``DataInput`` can hold this pointer.
    internal init(withRawPointer ptr: DataInputPtr) {
        self.pointer = ptr
    }
    
    /// Get box id
    func getBoxId() -> BoxId {
        var boxIdPtr: BoxIdPtr?
        ergo_wallet_data_input_box_id(self.pointer, &boxIdPtr)
        return BoxId(withRawPointer: boxIdPtr!)
    }
        
    deinit {
        ergo_wallet_data_input_delete(self.pointer)
    }
}

/// An ordered collection of `BlockHeader`s
class DataInputs {
    internal var pointer: DataInputsPtr
    
    /// Create an empty collection
    init() {
        self.pointer = DataInputs.initRawPtrEmpty()
    }
    
    /// Takes ownership of an existing ``DataInputsPtr``. Note: we must ensure that no other instance
    /// of ``DataInputs`` can hold this pointer.
    init(withRawPointer ptr: DataInputsPtr) {
        self.pointer = ptr
    }
    
    /// Use the C-API to create an empty collection and return the raw pointer that points to this
    /// collection.
    private static func initRawPtrEmpty() -> DataInputsPtr {
        var dataInputsPtr: DataInputsPtr?
        ergo_wallet_data_inputs_new(&dataInputsPtr)
        return dataInputsPtr!
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_wallet_data_inputs_len(self.pointer)
    }
    
    /// Returns the ``DataInput`` at location `index` if it exists.
    func get(index: UInt) -> DataInput? {
        var ptr: DataInputPtr?
        let res = ergo_wallet_data_inputs_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return DataInput(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    /// Add a ``DataInput`` to the end of the collection.
    func add(dataInput: DataInput) {
        ergo_wallet_data_inputs_add(dataInput.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_data_inputs_delete(self.pointer)
    }
}

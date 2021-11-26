import Foundation
import ErgoLibC

class DataInput {
    internal var pointer: DataInputPtr
    
    init(withBoxId: BoxId) {
        var ptr: DataInputPtr?
        ergo_wallet_data_input_new(withBoxId.pointer, &ptr)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: DataInputPtr) {
        self.pointer = ptr
    }
    
    func getBoxId() -> BoxId {
        var boxIdPtr: BoxIdPtr?
        ergo_wallet_data_input_box_id(self.pointer, &boxIdPtr)
        return BoxId(withPtr: boxIdPtr!)
    }
        
    deinit {
        ergo_wallet_data_input_delete(self.pointer)
    }
}

class DataInputs {
    internal var pointer: DataInputsPtr
    
    init() throws {
        self.pointer = try DataInputs.initEmpty()
    }
    
    init(withPtr ptr: DataInputsPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() throws -> DataInputsPtr {
        var dataInputsPtr: DataInputsPtr?
        let error = ergo_wallet_data_inputs_new(&dataInputsPtr)
        try checkError(error)
        return dataInputsPtr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_data_inputs_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> DataInput? {
        var dataInputPtr: DataInputPtr?
        let res = ergo_wallet_data_inputs_get(self.pointer, index, &dataInputPtr)
        try checkError(res.error)
        if res.is_some {
            return DataInput(withPtr: dataInputPtr!)
        } else {
            return nil
        }
    }
    
    func add(dataInput: DataInput) throws {
        let error = ergo_wallet_data_inputs_add(dataInput.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_data_inputs_delete(self.pointer)
    }
}

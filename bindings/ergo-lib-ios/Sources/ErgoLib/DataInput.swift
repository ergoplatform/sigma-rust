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
    
    init() {
        self.pointer = DataInputs.initEmpty()
    }
    
    init(withPtr ptr: DataInputsPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() -> DataInputsPtr {
        var dataInputsPtr: DataInputsPtr?
        ergo_wallet_data_inputs_new(&dataInputsPtr)
        return dataInputsPtr!
    }
    
    func len() -> UInt {
        return ergo_wallet_data_inputs_len(self.pointer)
    }
    
    func get(index: UInt) -> DataInput? {
        var dataInputPtr: DataInputPtr?
        let res = ergo_wallet_data_inputs_get(self.pointer, index, &dataInputPtr)
        assert(res.error == nil)
        if res.is_some {
            return DataInput(withPtr: dataInputPtr!)
        } else {
            return nil
        }
    }
    
    func add(dataInput: DataInput) {
        ergo_wallet_data_inputs_add(dataInput.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_data_inputs_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC

class ByteArray {
    internal var pointer: ByteArrayPtr
    
    init(fromBytes : [UInt8]) throws {
        var ptr: ByteArrayPtr?
        let error = ergo_lib_byte_array_from_raw_parts(fromBytes, UInt(fromBytes.count), &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``ByteArrayPtr``. Note: we must ensure that no other instance
    /// of ``ByteArray`` can hold this pointer.
    internal init(withRawPointer ptr: ByteArrayPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_lib_byte_array_delete(self.pointer)
    }
}

class ByteArrays {
    internal var pointer: ByteArraysPtr
    
    init() {
        var ptr: ByteArraysPtr?
        ergo_lib_byte_arrays_new(&ptr)
        self.pointer = ptr!
    }
    
    func len() -> UInt {
        return ergo_lib_byte_arrays_len(self.pointer)
    }
    
    func get(index: UInt) -> ByteArray? {
        var ptr: ByteArrayPtr?
        let res = ergo_lib_byte_arrays_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return ByteArray(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    func add(byteArray: ByteArray) {
        ergo_lib_byte_arrays_add(byteArray.pointer, self.pointer)
    }
        
    deinit {
        ergo_lib_byte_arrays_delete(self.pointer)
    }
}

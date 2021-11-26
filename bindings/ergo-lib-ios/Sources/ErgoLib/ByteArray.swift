import Foundation
import ErgoLibC

class ByteArray {
    internal var pointer: ByteArrayPtr
    
    init(fromBytes : [UInt8]) throws {
        var ptr: ByteArrayPtr?
        let error = ergo_wallet_byte_array_from_raw_parts(fromBytes, UInt(fromBytes.count), &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    deinit {
        ergo_wallet_byte_array_delete(self.pointer)
    }
}

class ByteArrays {
    internal var pointer: ByteArraysPtr
    
    init() {
        var ptr: BlockHeadersPtr?
        ergo_wallet_byte_arrays_new(&ptr)
        self.pointer = ptr!
    }
    
    func len() -> UInt {
        return ergo_wallet_byte_arrays_len(self.pointer)
    }
    
    func get(index: UInt) -> BlockHeader? {
        var blockHeaderPtr: BlockHeaderPtr?
        let res = ergo_wallet_byte_arrays_get(self.pointer, index, &blockHeaderPtr)
        assert(res.error == nil)
        if res.is_some {
            return BlockHeader(withPtr: blockHeaderPtr!)
        } else {
            return nil
        }
    }
    
    func add(byteArray: ByteArray) {
        ergo_wallet_byte_arrays_add(byteArray.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_byte_arrays_delete(self.pointer)
    }
}

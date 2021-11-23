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
    
    init() throws {
        var ptr: BlockHeadersPtr?
        let error = ergo_wallet_byte_arrays_new(&ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_byte_arrays_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> BlockHeader? {
        var blockHeaderPtr: BlockHeaderPtr?
        let res = ergo_wallet_byte_arrays_get(self.pointer, index, &blockHeaderPtr)
        try checkError(res.error)
        if res.is_some {
            return BlockHeader(withPtr: blockHeaderPtr!)
        } else {
            return nil
        }
    }
    
    func add(byteArray: ByteArray) throws {
        let error = ergo_wallet_byte_arrays_add(byteArray.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_byte_arrays_delete(self.pointer)
    }
}


import Foundation
import ErgoLibC

class Constant {
    internal var pointer: ConstantPtr
    
    init(withInt32: Int32) throws {
        var ptr: ConstantPtr?
        let error = ergo_wallet_constant_from_i32(&ptr, withInt32)
        try checkError(error)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: ConstantPtr) {
        self.pointer = ptr
    }
    
    
    deinit {
        ergo_wallet_constant_delete(self.pointer)
    }
}

extension Constant: Equatable {
    static func ==(lhs: Constant, rhs: Constant) -> Bool {
        ergo_wallet_constant_eq(lhs.pointer, rhs.pointer)
    }
}

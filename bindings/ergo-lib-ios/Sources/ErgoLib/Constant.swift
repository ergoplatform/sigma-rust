
import Foundation
import ErgoLibC

class Constant {
    internal var pointer: ConstantPtr
    
    internal init(withPtr ptr: ConstantPtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_wallet_constant_delete(self.pointer)
    }
}

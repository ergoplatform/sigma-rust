
import Foundation
import ErgoLibC

/// Blockchain parameters that can be changed by voting
class Parameters {
    internal var pointer: ParametersPtr

    /// Create default parameters
    init() {
        var ptr: ParametersPtr?
        ergo_lib_parameters_default(&ptr)
        self.pointer = ptr!
    }

    deinit {
        ergo_lib_parameters_delete(self.pointer)
    }
}

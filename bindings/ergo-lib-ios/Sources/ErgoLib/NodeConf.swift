import Foundation
import ErgoLibC

class NodeConf {
    internal var pointer: NodeConfPtr

    internal init(withRawPointer ptr: NodeConfPtr) {
        self.pointer = ptr
    }

    deinit {
        ergo_lib_node_conf_delete(self.pointer)
    }
}

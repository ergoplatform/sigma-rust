import Foundation
import ErgoLibC

class NodeInfo {
    internal var pointer: NodeInfoPtr

    internal init(withRawPointer ptr: NodeInfoPtr) {
        self.pointer = ptr
    }

    deinit {
        ergo_lib_node_info_delete(self.pointer)
    }
}

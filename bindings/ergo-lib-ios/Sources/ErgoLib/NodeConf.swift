import Foundation
import ErgoLibC

class NodeConf {
    internal var pointer: NodeConfPtr

    internal init(withRawPointer ptr: NodeConfPtr) {
        self.pointer = ptr
    }

    init(withAddrString addrStr: String) throws {
        var ptr: NodeConfPtr?
        let error = addrStr.withCString { cs in
            ergo_lib_node_conf_from_addr(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }

    deinit {
        ergo_lib_node_conf_delete(self.pointer)
    }
}

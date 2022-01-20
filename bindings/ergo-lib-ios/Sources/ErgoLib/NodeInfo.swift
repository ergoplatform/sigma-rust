import Foundation
import ErgoLibC

class NodeInfo: FromRawPtr {
    internal var pointer: NodeInfoPtr

    internal init(withRawPointer ptr: NodeInfoPtr) {
        self.pointer = ptr
    }

    static func fromRawPtr(ptr: UnsafeRawPointer) -> Self {
        return NodeInfo(withRawPointer: OpaquePointer(ptr)) as! Self
    }

    deinit {
        ergo_lib_node_info_delete(self.pointer)
    }

    func getName() -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_lib_node_info_get_name(self.pointer, &cStr)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }

}

protocol FromRawPtr {
    static func fromRawPtr(ptr: UnsafeRawPointer) -> Self
}

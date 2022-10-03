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

    func getName() -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_lib_node_info_get_name(self.pointer, &cStr)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }

    func isAtLeastVersion40100() -> Bool {
        return ergo_lib_node_info_is_at_least_version_4_0_100(self.pointer)
    }

    deinit {
        ergo_lib_node_info_delete(self.pointer)
    }
}

protocol FromRawPtr {
    static func fromRawPtr(ptr: UnsafeRawPointer) -> Self
}

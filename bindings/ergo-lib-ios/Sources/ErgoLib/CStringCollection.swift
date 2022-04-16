import Foundation
import ErgoLibC

class CStringCollection: FromRawPtr {
    internal var pointer: CStringCollectionPtr

    internal init(withRawPointer ptr: CStringCollectionPtr) {
        self.pointer = ptr
    }

    static func fromRawPtr(ptr: UnsafeRawPointer) -> Self {
        return CStringCollection(withRawPointer: OpaquePointer(ptr)) as! Self
    }

    func getPtr() -> UnsafePointer<UnsafePointer<CChar>?>! {
        return ergo_lib_c_string_collection_get_ptr(self.pointer)
    }
    
    func getLength() -> UInt {
        return ergo_lib_c_string_collection_get_length(self.pointer)
    }
    
    func toArray() -> [String] {
        let ptr = self.getPtr()
        var res: [String] = []
        if self.getLength() == UInt(0) {
            return res;
        }
        for _ in 0 ..< self.getLength() {
            if var ptr = ptr {
                if let s = ptr.pointee {
                    let str = String(cString: s)
                    res.append(str)
                    print(str)
                    ptr = ptr.advanced(by: 1)
                }
            } else {
                break
            }
        }
        return res
    }

    deinit {
        ergo_lib_c_string_collection_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC

/// The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around.
class ErgoTree {
    internal var pointer: ErgoTreePtr
    
    /// Decode from encoded serialized ``ErgoTree``
    init(fromBytes : [UInt8]) throws {
        var ptr: ErgoTreePtr?
        let error = ergo_lib_ergo_tree_from_bytes(
            fromBytes,
            UInt(fromBytes.count),
            &ptr
        )
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Decode from base16 encoded serialized ``ErgoTree``
    init(fromBase16EncodedString : String) throws {
        var ptr: ErgoTreePtr?
        let error = fromBase16EncodedString.withCString { cs in
            ergo_lib_ergo_tree_from_base16_bytes(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``ErgoTreePtr``. Note: we must ensure that no other instance
    /// of ``ErgoTree`` can hold this pointer.
    init(withRawPointer ptr: ErgoTreePtr) {
        self.pointer = ptr
    }
    
    /// Convert to serialized bytes.
    func toBytes() throws -> [UInt8] {
        let res = ergo_lib_ergo_tree_bytes_len(self.pointer)
        try checkError(res.error)
        var bytes = Array.init(repeating: UInt8(0), count: Int(res.value))
        let error = ergo_lib_ergo_tree_to_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    /// Convert to base16-encoded serialized bytes
    func toBase16EncodedString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_lib_ergo_tree_to_base16_bytes(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    /// Returns the number of constants stored in the serialized ``ErgoTree`` or throws error if the
    /// parsing of constants failed
    func constantsLength() throws -> UInt {
        let res = ergo_lib_ergo_tree_constants_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    /// Return constant with given index (as stored in serialized ErgoTree) if it exists. Throws if
    /// constant parsing failed.
    func getConstant(index: UInt) throws -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_lib_ergo_tree_get_constant(self.pointer, index, &constantPtr)
        try checkError(res.error)
        if res.is_some {
            return Constant(withRawPointer: constantPtr!)
        } else {
            return nil
        }
    }
    
    /// Replace the constant of the ``ErgoTree`` with the given `constant` at position `index`.
    /// Throws if no constant exists at `index`.
    func withConstant(index: UInt, constant: Constant) throws {
        var newErgoTreePtr: ErgoTreePtr?
        let error = ergo_lib_ergo_tree_with_constant(self.pointer, index, constant.pointer, &newErgoTreePtr)
        try checkError(error)
        
        // Delete the old ErgoTree and point to new instance
        ergo_lib_ergo_tree_delete(self.pointer)
        self.pointer = newErgoTreePtr!
    }
    
    /// Serialized proposition expression of SigmaProp type with ConstantPlaceholder nodes instead of
    /// Constant nodes.
    func toTemplateBytes() throws -> [UInt8] {
        let res = ergo_lib_ergo_tree_template_bytes_len(self.pointer)
        try checkError(res.error)
        var bytes = Array.init(repeating: UInt8(0), count: Int(res.value))
        let error = ergo_lib_ergo_tree_template_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    deinit {
        ergo_lib_ergo_tree_delete(self.pointer)
    }
}

extension ErgoTree: Equatable {
    static func ==(lhs: ErgoTree, rhs: ErgoTree) -> Bool {
        ergo_lib_ergo_tree_eq(lhs.pointer, rhs.pointer)
    }
}

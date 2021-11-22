import Foundation
import ErgoLibC

class BoxId {
    internal var pointer: BoxIdPtr
    
    init(withString str: String) throws {
        self.pointer = try BoxId.fromString(str: str)
    }
    
    internal init(withPtr ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    func toBytes() throws -> [UInt8] {
        var bytes = Array.init(repeating: UInt8(0), count: 32)
        let error = ergo_wallet_box_id_to_bytes(self.pointer, &bytes)
        try checkError(error)
        return bytes
    }
    
    func toString() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_box_id_to_str(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    private static func fromString(str: String) throws -> BoxIdPtr {
        var ptr: BoxIdPtr?
        let error = str.withCString { cs in
            ergo_wallet_box_id_from_str(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_box_id_delete(self.pointer)
    }
}

class BoxValue {
    internal var pointer: BoxValuePtr
    
    init(fromInt64 : Int64) throws {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_box_value_from_i64(fromInt64, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withPtr ptr: BoxValuePtr) {
        self.pointer = ptr
    }
    
    func toInt64() throws -> Int64 {
        let res = ergo_wallet_box_value_as_i64(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    deinit {
        ergo_wallet_box_value_delete(self.pointer)
    }
}

class ErgoBoxCandidate {
    internal var pointer: ErgoBoxCandidatePtr

    internal init(withRawPointer pointer: ErgoBoxCandidatePtr) {
        self.pointer = pointer
    }
    
    func getRegisterValue(registerId: NonMandatoryRegisterId) throws -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_wallet_ergo_tree_register_value(self.pointer, registerId.rawValue, &constantPtr)
        try checkError(res.error)
        if res.is_some {
            return Constant(withPtr: constantPtr!)
        } else {
            return nil
        }
    }
    
    func getCreationHeight() throws -> UInt32 {
        let res = ergo_wallet_ergo_box_candidate_creation_height(self.pointer)
        try checkError(res.error)
        return res.value
    }

    func getTokens() throws -> Tokens {
        var tokensPtr: TokensPtr?
        let error = ergo_wallet_ergo_box_candidate_tokens(self.pointer, &tokensPtr)
        try checkError(error)
        return Tokens(withPtr: tokensPtr!)
    }
    
    func getErgoTree() throws -> ErgoTree {
        var ergoTreePtr: ErgoTreePtr?
        let error = ergo_wallet_ergo_box_candidate_ergo_tree(self.pointer, &ergoTreePtr)
        try checkError(error)
        return ErgoTree(withPtr: ergoTreePtr!)
    }
    
    func getBoxValue() throws -> BoxValue {
        var boxValuePtr: BoxValuePtr?
        let error = ergo_wallet_ergo_box_candidate_box_value(self.pointer, &boxValuePtr)
        try checkError(error)
        return BoxValue(withPtr: boxValuePtr!)
    }
    
    deinit {
        ergo_wallet_ergo_box_candidate_delete(self.pointer)
    }
}

enum NonMandatoryRegisterId: UInt8 {
    /// id for R4 register
    case R4 = 4
    /// id for R5 register
    case R5 = 5
    /// id for R6 register
    case R6 = 6
    /// id for R7 register
    case R7 = 7
    /// id for R8 register
    case R8 = 8
    /// id for R9 register
    case R9 = 9
}

class ErgoBoxCandidates {
    internal var pointer: ErgoBoxCandidatesPtr
    
    init() throws {
        self.pointer = try ErgoBoxCandidates.initEmpty()
    }
    
    init(withRawPointer ptr: ErgoBoxCandidatesPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() throws -> ErgoBoxCandidatesPtr {
        var ptr: ErgoBoxCandidatesPtr?
        let error = ergo_wallet_ergo_box_candidates_new(&ptr)
        try checkError(error)
        return ptr!
    }
    
    func len() throws -> UInt {
        let res = ergo_wallet_ergo_box_candidates_len(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func get(index: UInt) throws -> ErgoBoxCandidate? {
        var dataInputPtr: DataInputPtr?
        let res = ergo_wallet_ergo_box_candidates_get(self.pointer, index, &dataInputPtr)
        try checkError(res.error)
        if res.is_some {
            return ErgoBoxCandidate(withRawPointer: dataInputPtr!)
        } else {
            return nil
        }
    }
    
    func add(ergoBoxCandidate: ErgoBoxCandidate) throws {
        let error = ergo_wallet_ergo_box_candidates_add(ergoBoxCandidate.pointer, self.pointer)
        try checkError(error)
    }
        
    deinit {
        ergo_wallet_ergo_box_candidates_delete(self.pointer)
    }
}

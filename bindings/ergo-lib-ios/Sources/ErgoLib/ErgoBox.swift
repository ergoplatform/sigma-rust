import Foundation
import ErgoLibC
import SwiftyJSON

class BoxId {
    internal var pointer: BoxIdPtr
    
    init(withString str: String) throws {
        self.pointer = try BoxId.fromString(str: str)
    }
    
    internal init(withPtr ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    func toBytes() -> [UInt8] {
        var bytes = Array.init(repeating: UInt8(0), count: 32)
        ergo_wallet_box_id_to_bytes(self.pointer, &bytes)
        return bytes
    }
    
    func toString() -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_wallet_box_id_to_str(self.pointer, &cStr)
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

extension BoxId: Equatable {
    static func ==(lhs: BoxId, rhs: BoxId) -> Bool {
        ergo_wallet_box_id_eq(lhs.pointer, rhs.pointer)
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
    
    init(withRawPointer ptr: BoxValuePtr) {
        self.pointer = ptr
    }
    
    static func SAFE_USER_MIN() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_wallet_box_value_safe_user_min(&ptr)
        return BoxValue(withRawPointer: ptr!)
    }
    
    static func UNITS_PER_ERGO() -> Int64 {
        return ergo_wallet_box_value_units_per_ergo()
    }
    
    // Note that swift forbids integer overflow/underflow by default with hard crash.
    static func sumOf(boxValue0: BoxValue, boxValue1: BoxValue) throws -> BoxValue {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_box_value_sum_of(boxValue0.pointer, boxValue1.pointer, &ptr)
        try checkError(error)
        return BoxValue(withRawPointer: ptr!)
    }
    
    func toInt64() -> Int64 {
        return ergo_wallet_box_value_as_i64(self.pointer)
    }
    
    deinit {
        ergo_wallet_box_value_delete(self.pointer)
    }
}

extension BoxValue: Equatable {
    static func ==(lhs: BoxValue, rhs: BoxValue) -> Bool {
        ergo_wallet_box_value_eq(lhs.pointer, rhs.pointer)
    }
}

class ErgoBoxCandidate {
    internal var pointer: ErgoBoxCandidatePtr

    internal init(withRawPointer pointer: ErgoBoxCandidatePtr) {
        self.pointer = pointer
    }
    
    func getRegisterValue(registerId: NonMandatoryRegisterId) -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_wallet_ergo_box_candidate_register_value(self.pointer, registerId.rawValue, &constantPtr)
        assert(res.error == nil)
        if res.is_some {
            return Constant(withPtr: constantPtr!)
        } else {
            return nil
        }
    }
    
    func getCreationHeight() -> UInt32 {
        return ergo_wallet_ergo_box_candidate_creation_height(self.pointer)
    }

    func getTokens() -> Tokens {
        var tokensPtr: TokensPtr?
        ergo_wallet_ergo_box_candidate_tokens(self.pointer, &tokensPtr)
        return Tokens(withPtr: tokensPtr!)
    }
    
    func getErgoTree() -> ErgoTree {
        var ergoTreePtr: ErgoTreePtr?
        ergo_wallet_ergo_box_candidate_ergo_tree(self.pointer, &ergoTreePtr)
        return ErgoTree(withPtr: ergoTreePtr!)
    }
    
    func getBoxValue() -> BoxValue {
        var boxValuePtr: BoxValuePtr?
        ergo_wallet_ergo_box_candidate_box_value(self.pointer, &boxValuePtr)
        return BoxValue(withRawPointer: boxValuePtr!)
    }
    
    deinit {
        ergo_wallet_ergo_box_candidate_delete(self.pointer)
    }
}

extension ErgoBoxCandidate: Equatable {
    static func ==(lhs: ErgoBoxCandidate, rhs: ErgoBoxCandidate) -> Bool {
        ergo_wallet_ergo_box_candidate_eq(lhs.pointer, rhs.pointer)
    }
}

class ErgoBox{
    internal var pointer: ErgoBoxPtr

    init( boxValue: BoxValue,
          creationHeight: UInt32,
          contract: Contract,
          txId: TxId,
          index: UInt16,
          tokens: Tokens
    ) throws {
        var ptr: ErgoBoxPtr?
        let error = ergo_wallet_ergo_box_new(
            boxValue.pointer,
            creationHeight,
            contract.pointer,
            txId.pointer,
            index,
            tokens.pointer,
            &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withJson json: String) throws {
        var ptr: ErgoBoxPtr?
        let error = json.withCString { cs in
            ergo_wallet_ergo_box_from_json(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    internal init(withRawPointer pointer: ErgoBoxPtr) {
        self.pointer = pointer
    }
    
    func getBoxId() -> BoxId {
        var ptr: BoxIdPtr?
        ergo_wallet_ergo_box_id(self.pointer, &ptr)
        return BoxId(withPtr: ptr!)
    }
    
    func getCreationHeight() -> UInt32 {
        return ergo_wallet_ergo_box_creation_height(self.pointer)
    }
    
    func getTokens() -> Tokens {
        var tokensPtr: TokensPtr?
        ergo_wallet_ergo_box_tokens(self.pointer, &tokensPtr)
        return Tokens(withPtr: tokensPtr!)
    }
    
    func getErgoTree() -> ErgoTree {
        var ergoTreePtr: ErgoTreePtr?
        ergo_wallet_ergo_box_ergo_tree(self.pointer, &ergoTreePtr)
        return ErgoTree(withPtr: ergoTreePtr!)
    }
    
    func getBoxValue() -> BoxValue {
        var boxValuePtr: BoxValuePtr?
        ergo_wallet_ergo_box_value(self.pointer, &boxValuePtr)
        return BoxValue(withRawPointer: boxValuePtr!)
    }
    
    func getRegisterValue(registerId: NonMandatoryRegisterId) -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_wallet_ergo_box_register_value(self.pointer, registerId.rawValue, &constantPtr)
        assert(res.error == nil)
        if res.is_some {
            return Constant(withPtr: constantPtr!)
        } else {
            return nil
        }
    }
    
    func toJSON() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_ergo_box_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    func toJsonEIP12() throws -> JSON? {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_ergo_box_to_json_eip12(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return try str.data(using: .utf8, allowLossyConversion: false).map {
            try JSON(data: $0)
        }
    }
    
    deinit {
        ergo_wallet_ergo_box_delete(self.pointer)
    }
}

extension ErgoBox: Equatable {
    static func ==(lhs: ErgoBox, rhs: ErgoBox) -> Bool {
        ergo_wallet_ergo_box_eq(lhs.pointer, rhs.pointer)
    }
}

class ErgoBoxAssetsData {
    internal var pointer: ErgoBoxPtr

    init( boxValue: BoxValue,
          tokens: Tokens
    ) {
        var ptr: ErgoBoxAssetsDataPtr?
        ergo_wallet_ergo_box_assets_data_new(
            boxValue.pointer,
            tokens.pointer,
            &ptr)
        self.pointer = ptr!
    }
    
    internal init(withRawPointer pointer: ErgoBoxPtr) {
        self.pointer = pointer
    }
    
    func getBoxValue() -> BoxValue {
        var boxValuePtr: BoxValuePtr?
        ergo_wallet_ergo_box_assets_data_value(self.pointer, &boxValuePtr)
        return BoxValue(withRawPointer: boxValuePtr!)
    }
    
    func getTokens() -> Tokens {
        var tokensPtr: TokensPtr?
        ergo_wallet_ergo_box_assets_data_tokens(self.pointer, &tokensPtr)
        return Tokens(withPtr: tokensPtr!)
    }
    
    deinit {
        ergo_wallet_ergo_box_assets_data_delete(self.pointer)
    }
}

extension ErgoBoxAssetsData: Equatable {
    static func ==(lhs: ErgoBoxAssetsData, rhs: ErgoBoxAssetsData) -> Bool {
        ergo_wallet_ergo_box_assets_data_eq(lhs.pointer, rhs.pointer)
    }
}

class ErgoBoxAssetsDataList {
    internal var pointer: ErgoBoxAssetsDataListPtr
    
    init() {
        var ptr: ErgoBoxAssetsDataListPtr?
        ergo_wallet_ergo_box_assets_data_list_new(&ptr)
        self.pointer = ptr!
    }
    
    init(withRawPointer ptr: ErgoBoxAssetsDataListPtr) {
        self.pointer = ptr
    }
    
    func len() -> UInt {
        return ergo_wallet_ergo_box_assets_data_list_len(self.pointer)
    }
    
    func get(index: UInt) -> ErgoBoxAssetsData? {
        var ptr: ErgoBoxAssetsDataPtr?
        let res = ergo_wallet_ergo_box_assets_data_list_get(self.pointer, index, &ptr)
        assert(res.error == nil)
        if res.is_some {
            return ErgoBoxAssetsData(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    func add(ergoBoxAssetData: ErgoBoxAssetsData) {
        ergo_wallet_ergo_box_assets_data_list_add(ergoBoxAssetData.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_ergo_box_assets_data_list_delete(self.pointer)
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
    
    init() {
        self.pointer = ErgoBoxCandidates.initEmpty()
    }
    
    init(withRawPointer ptr: ErgoBoxCandidatesPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() -> ErgoBoxCandidatesPtr {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_ergo_box_candidates_new(&ptr)
        return ptr!
    }
    
    func len() -> UInt {
        return ergo_wallet_ergo_box_candidates_len(self.pointer)
    }
    
    func get(index: UInt) -> ErgoBoxCandidate? {
        var dataInputPtr: DataInputPtr?
        let res = ergo_wallet_ergo_box_candidates_get(self.pointer, index, &dataInputPtr)
        assert(res.error == nil)
        if res.is_some {
            return ErgoBoxCandidate(withRawPointer: dataInputPtr!)
        } else {
            return nil
        }
    }
    
    func add(ergoBoxCandidate: ErgoBoxCandidate) {
        ergo_wallet_ergo_box_candidates_add(ergoBoxCandidate.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_ergo_box_candidates_delete(self.pointer)
    }
}

class ErgoBoxes {
    internal var pointer: ErgoBoxesPtr
    
    init() {
        self.pointer = ErgoBoxes.initEmpty()
    }
    
    init(fromJSON: Any) throws {
        let json = JSON(fromJSON)
        if let arr = json.array {
            let boxes = try arr.map{ elem -> ErgoBox in
                if let jsonStr = elem.rawString() {
                    return try ErgoBox(withJson: jsonStr);
                } else {
                    throw WalletError.walletCError(reason: "Ergoboxes.init(fromJSON): cannot cast array element to raw JSON string")
                }
            }
            self.pointer = ErgoBoxes.initEmpty()
            for ergoBox in boxes {
                self.add(ergoBox: ergoBox)
            }
        } else {
            throw WalletError.walletCError(reason: "Ergoboxes.init(fromJSON): expected [JSON]")
        }
    }
    
    init(withRawPointer ptr: ErgoBoxesPtr) {
        self.pointer = ptr
    }
    
    private static func initEmpty() -> ErgoBoxesPtr {
        var ptr: ErgoBoxesPtr?
        ergo_wallet_ergo_boxes_new(&ptr)
        return ptr!
    }
    
    func len() -> UInt {
        return ergo_wallet_ergo_boxes_len(self.pointer)
    }
    
    func get(index: UInt) -> ErgoBox? {
        var dataInputPtr: DataInputPtr?
        let res = ergo_wallet_ergo_boxes_get(self.pointer, index, &dataInputPtr)
        assert(res.error == nil)
        if res.is_some {
            return ErgoBox(withRawPointer: dataInputPtr!)
        } else {
            return nil
        }
    }
    
    func add(ergoBox: ErgoBox) {
        ergo_wallet_ergo_boxes_add(ergoBox.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_ergo_boxes_delete(self.pointer)
    }
}

/// Box (aka coin, or an unspent output) is a basic concept of a UTXO-based cryptocurrency.
/// In Bitcoin, such an object is associated with some monetary value (arbitrary,
/// but with predefined precision, so we use integer arithmetic to work with the value),
/// and also a guarding script (aka proposition) to protect the box from unauthorized opening.
///
/// In other way, a box is a state element locked by some proposition (ErgoTree).
///
/// In Ergo, box is just a collection of registers, some with mandatory types and semantics,
/// others could be used by applications in any way.
/// We add additional fields in addition to amount and proposition~(which stored in the registers R0 and R1).
/// Namely, register R2 contains additional tokens (a sequence of pairs (token identifier, value)).
/// Register R3 contains height when block got included into the blockchain and also transaction
/// identifier and box index in the transaction outputs.
/// Registers R4-R9 are free for arbitrary usage.
///
/// A transaction is unsealing a box. As a box can not be open twice, any further valid transaction
//! can not be linked to the same box.
import Foundation
import ErgoLibC

/// Box id (32-byte digest)
class BoxId {
    internal var pointer: BoxIdPtr
    
    /// Parse box id (32 byte digest) from base16-encoded string
    init(withString str: String) throws {
        var ptr: BoxIdPtr?
        let error = str.withCString { cs in
            ergo_wallet_box_id_from_str(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``BoxIdPtr``. Note: we must ensure that no other instance
    /// of ``BoxId`` can hold this pointer.
    internal init(withRawPointer ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    /// Returns byte array (32 bytes) representation
    func toBytes() -> [UInt8] {
        var bytes = Array.init(repeating: UInt8(0), count: 32)
        ergo_wallet_box_id_to_bytes(self.pointer, &bytes)
        return bytes
    }
     
    /// Base16 encoded string
    func toString() -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_wallet_box_id_to_str(self.pointer, &cStr)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
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

/// Box value in nanoERGs with bound checks
class BoxValue {
    internal var pointer: BoxValuePtr
    
    /// Create from ``Int64`` with bounds check
    init(fromInt64 : Int64) throws {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_box_value_from_i64(fromInt64, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(withRawPointer ptr: BoxValuePtr) {
        self.pointer = ptr
    }
    
    /// Recommended (safe) minimal box value to use in case box size estimation is unavailable.
    /// Allows box size upto 2777 bytes with current min box value per byte of 360 nanoERGs
    static func SAFE_USER_MIN() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_wallet_box_value_safe_user_min(&ptr)
        return BoxValue(withRawPointer: ptr!)
    }
    
    /// Number of units inside one ERGO (i.e. one ERG using nano ERG representation)
    static func UNITS_PER_ERGO() -> Int64 {
        return ergo_wallet_box_value_units_per_ergo()
    }
    
    /// Create a new box value which is the sum of the arguments, throwing error if value is out of
    /// bounds. Note that swift forbids integer overflow/underflow by default with hard crash, which
    /// is not desirable here.
    static func sumOf(boxValue0: BoxValue, boxValue1: BoxValue) throws -> BoxValue {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_box_value_sum_of(boxValue0.pointer, boxValue1.pointer, &ptr)
        try checkError(error)
        return BoxValue(withRawPointer: ptr!)
    }
    
    /// Get value as ``Int64``
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

/// Contains the same fields as ``ErgoBox``, except for transaction id and index, that will be
/// calculated after full transaction formation.  Use ``ErgoBoxCandidateBuilder`` to create an
/// instance.
class ErgoBoxCandidate {
    internal var pointer: ErgoBoxCandidatePtr

    /// Takes ownership of an existing ``ErgoBoxCandidatePtr``. Note: we must ensure that no other
    /// instance of ``ErgoBoxCandidate`` can hold this pointer.
    internal init(withRawPointer pointer: ErgoBoxCandidatePtr) {
        self.pointer = pointer
    }
    
    /// Returns value (ErgoTree constant) stored in the register or `nil` if the register is empty
    func getRegisterValue(registerId: NonMandatoryRegisterId) -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_wallet_ergo_box_candidate_register_value(self.pointer, registerId.rawValue, &constantPtr)
        assert(res.error == nil)
        if res.is_some {
            return Constant(withRawPointer: constantPtr!)
        } else {
            return nil
        }
    }
    
    /// Get box creation height
    func getCreationHeight() -> UInt32 {
        return ergo_wallet_ergo_box_candidate_creation_height(self.pointer)
    }

    /// Get tokens for box
    func getTokens() -> Tokens {
        var tokensPtr: TokensPtr?
        ergo_wallet_ergo_box_candidate_tokens(self.pointer, &tokensPtr)
        return Tokens(withPtr: tokensPtr!)
    }
    
    /// Get ergo tree for box
    func getErgoTree() -> ErgoTree {
        var ergoTreePtr: ErgoTreePtr?
        ergo_wallet_ergo_box_candidate_ergo_tree(self.pointer, &ergoTreePtr)
        return ErgoTree(withRawPointer: ergoTreePtr!)
    }
    
    /// Get box value
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

/// Ergo box, that is taking part in some transaction on the chain Differs with ``ErgoBoxCandidate``
/// by added transaction id and an index in the input of that transaction
class ErgoBox{
    internal var pointer: ErgoBoxPtr

    /// Make a new box.
    /// - Parameters
    ///  - `boxValue`: amount of money associated with the box
    ///  - `creationHeight`: height when a transaction containing the box is created.
    ///  - `contract`: guarding contract(``Contract``), which should be evaluated to true in order
    ///    to open(spend) this box
    ///  - `txId`: transaction id in which this box was "created" (participated in outputs)
    ///  - `index`: index (in outputs) in the transaction
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
    
    /// Parse from JSON.  Supports Ergo Node/Explorer API and box values and token amount encoded as
    /// strings.
    init(withJson json: String) throws {
        var ptr: ErgoBoxPtr?
        let error = json.withCString { cs in
            ergo_wallet_ergo_box_from_json(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``ErgoBoxPtr``. Note: we must ensure that no other instance
    /// of ``ErgoBox`` can hold this pointer.
    internal init(withRawPointer pointer: ErgoBoxPtr) {
        self.pointer = pointer
    }
    
    /// Get box id
    func getBoxId() -> BoxId {
        var ptr: BoxIdPtr?
        ergo_wallet_ergo_box_id(self.pointer, &ptr)
        return BoxId(withRawPointer: ptr!)
    }
    
    /// Get box creation height
    func getCreationHeight() -> UInt32 {
        return ergo_wallet_ergo_box_creation_height(self.pointer)
    }
    
    /// Get tokens for box
    func getTokens() -> Tokens {
        var tokensPtr: TokensPtr?
        ergo_wallet_ergo_box_tokens(self.pointer, &tokensPtr)
        return Tokens(withPtr: tokensPtr!)
    }
    
    /// Get ergo tree for box
    func getErgoTree() -> ErgoTree {
        var ergoTreePtr: ErgoTreePtr?
        ergo_wallet_ergo_box_ergo_tree(self.pointer, &ergoTreePtr)
        return ErgoTree(withRawPointer: ergoTreePtr!)
    }
    
    /// Get box value
    func getBoxValue() -> BoxValue {
        var boxValuePtr: BoxValuePtr?
        ergo_wallet_ergo_box_value(self.pointer, &boxValuePtr)
        return BoxValue(withRawPointer: boxValuePtr!)
    }
    
    /// Returns value (ErgoTree constant) stored in the register or `nil` if the register is empty
    func getRegisterValue(registerId: NonMandatoryRegisterId) -> Constant? {
        var constantPtr: ConstantPtr?
        let res = ergo_wallet_ergo_box_register_value(self.pointer, registerId.rawValue, &constantPtr)
        assert(res.error == nil)
        if res.is_some {
            return Constant(withRawPointer: constantPtr!)
        } else {
            return nil
        }
    }
    
    /// JSON representation as text (compatible with Ergo Node/Explorer API, numbers are encoded as numbers)
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
    
    /// JSON representation according to EIP-12 <https://github.com/ergoplatform/eips/pull/23>
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

/// Pair of (value, tokens) for a box
class ErgoBoxAssetsData {
    internal var pointer: ErgoBoxAssetsDataPtr

    /// Create new instance
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
    
    /// Takes ownership of an existing ``ErgoBoxAssetsDataPtr``. Note: we must ensure that no other
    /// instance of ``ErgoBoxAssetsData`` can hold this pointer.
    internal init(withRawPointer pointer: ErgoBoxAssetsDataPtr) {
        self.pointer = pointer
    }
    
    /// Value part of the box
    func getBoxValue() -> BoxValue {
        var boxValuePtr: BoxValuePtr?
        ergo_wallet_ergo_box_assets_data_value(self.pointer, &boxValuePtr)
        return BoxValue(withRawPointer: boxValuePtr!)
    }
    
    /// Tokens part of the box
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

/// An ordered collection of ``ErgoBoxAssetsData``s
class ErgoBoxAssetsDataList {
    internal var pointer: ErgoBoxAssetsDataListPtr
    
    /// Create an empty collection
    init() {
        var ptr: ErgoBoxAssetsDataListPtr?
        ergo_wallet_ergo_box_assets_data_list_new(&ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``ErgoBoxAssetsDataListPtr``. Note: we must ensure that no other
    /// instance of ``ErgoBoxAssetsDataList`` can hold this pointer.
    init(withRawPointer ptr: ErgoBoxAssetsDataListPtr) {
        self.pointer = ptr
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_wallet_ergo_box_assets_data_list_len(self.pointer)
    }
    
    /// Returns the ``ErgoBoxAssetsData`` at location `index` if it exists.
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
    
    /// Add a ``ErgoBoxAssetsData`` to the end of the collection.
    func add(ergoBoxAssetData: ErgoBoxAssetsData) {
        ergo_wallet_ergo_box_assets_data_list_add(ergoBoxAssetData.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_ergo_box_assets_data_list_delete(self.pointer)
    }
}

/// Type for representing box registers R4 - R9
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

/// An ordered collection of ``ErgoBoxCandidate``s
class ErgoBoxCandidates {
    internal var pointer: ErgoBoxCandidatesPtr
    
    /// Create an empty collection
    init() {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_ergo_box_candidates_new(&ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``ErgoBoxCandidatesPtr``. Note: we must ensure that no other
    /// instance of ``ErgoBoxCandidates`` can hold this pointer.
    init(withRawPointer ptr: ErgoBoxCandidatesPtr) {
        self.pointer = ptr
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_wallet_ergo_box_candidates_len(self.pointer)
    }
    
    /// Returns the ``ErgoBoxCandidates`` at location `index` if it exists.
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
    
    /// Add a ``ErgoBoxCandidates`` to the end of the collection.
    func add(ergoBoxCandidate: ErgoBoxCandidate) {
        ergo_wallet_ergo_box_candidates_add(ergoBoxCandidate.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_ergo_box_candidates_delete(self.pointer)
    }
}

/// An ordered collection of ``ErgoBox``s
class ErgoBoxes {
    internal var pointer: ErgoBoxesPtr
    
    init() {
        self.pointer = ErgoBoxes.initRawPtrEmpty()
    }
    
    /// Parse ``ErgoBox`` array from JSON (Node API)
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
            self.pointer = ErgoBoxes.initRawPtrEmpty()
            for ergoBox in boxes {
                self.add(ergoBox: ergoBox)
            }
        } else {
            throw WalletError.walletCError(reason: "Ergoboxes.init(fromJSON): expected [JSON]")
        }
    }
    
    /// Takes ownership of an existing ``ErgoBoxesPtr``. Note: we must ensure that no other
    /// instance of ``ErgoBoxes`` can hold this pointer.
    init(withRawPointer ptr: ErgoBoxesPtr) {
        self.pointer = ptr
    }
    
    /// Use the C-API to create an empty collection and return the raw pointer that points to this
    /// collection.
    private static func initRawPtrEmpty() -> ErgoBoxesPtr {
        var ptr: ErgoBoxesPtr?
        ergo_wallet_ergo_boxes_new(&ptr)
        return ptr!
    }
    
    /// Return the length of the collection
    func len() -> UInt {
        return ergo_wallet_ergo_boxes_len(self.pointer)
    }
    
    /// Returns the ``BlockHeader`` at location `index` if it exists.
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
    
    /// Add a ``ErgoBox`` to the end of the collection.
    func add(ergoBox: ErgoBox) {
        ergo_wallet_ergo_boxes_add(ergoBox.pointer, self.pointer)
    }
        
    deinit {
        ergo_wallet_ergo_boxes_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC

/// ``ErgoBoxCandidate`` builder
class ErgoBoxCandidateBuilder {
    internal var pointer: ErgoBoxCandidateBuilderPtr
    /// Create builder with required box parameters.
    /// - Parameters
    ///  - `boxValue`:  amount of money associated with the box
    ///  - `contract`: guarding contract([`Contract`]), which should be evaluated to true in order
    ///    to open(spend) this box
    ///  - `creationHeight`: height when a transaction containing the box is created.
    /// It should not exceed height of the block, containing the transaction with this box.
    init(boxValue: BoxValue, contract: Contract, creationHeight: UInt32) {
        var ptr: ErgoBoxCandidateBuilderPtr?
        ergo_lib_ergo_box_candidate_builder_new(
            boxValue.pointer,
            contract.pointer,
            creationHeight,
            &ptr
        )
        self.pointer = ptr!
    }
    
    /// Set minimal value (per byte of the serialized box size)
    func setMinBoxValuePerByte(minBoxValuePerByte: UInt32) -> ErgoBoxCandidateBuilder {
        ergo_lib_ergo_box_candidate_builder_set_min_box_value_per_byte(
            self.pointer,
            minBoxValuePerByte
        )
        return self
    }
    
    /// Get minimal value (per byte of the serialized box size)
    func getMinBoxValuePerByte() -> UInt32 {
        ergo_lib_ergo_box_candidate_builder_min_box_value_per_byte(self.pointer)
    }
    
    /// Set new box value
    func setValue(boxValue: BoxValue) -> ErgoBoxCandidateBuilder {
        ergo_lib_ergo_box_candidate_builder_set_value(
            self.pointer,
            boxValue.pointer
        )
        return self
    }
    
    /// Get box value
    func getValue() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_lib_ergo_box_candidate_builder_value(self.pointer, &ptr)
        return BoxValue(withRawPointer: ptr!)
    }
    
    /// Calculate serialized box size(in bytes)
    func calcBoxSizeBytes() throws -> UInt {
        let res = ergo_lib_ergo_box_candidate_builder_calc_box_size_bytes(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    /// Calculate minimal box value for the current box serialized size(in bytes)
    func calcMinBoxValue() throws -> BoxValue {
        var ptr: BoxValuePtr?
        let error = ergo_lib_ergo_box_candidate_calc_min_box_value(self.pointer, &ptr)
        try checkError(error)
        return BoxValue(withRawPointer: ptr!)
    }
    
    /// Set register with a given id (R4-R9) to the given value
    func setRegisterValue(
        registerId: NonMandatoryRegisterId,
        constant: Constant
    ) -> ErgoBoxCandidateBuilder {
        ergo_lib_ergo_box_candidate_builder_set_register_value(
            self.pointer,
            registerId.rawValue,
            constant.pointer
        )
        return self
    }
    
    /// Returns register value for the given register id (R4-R9), or None if the register is empty
    func getRegisterValue(registerId: NonMandatoryRegisterId) -> Constant? {
        var ptr: ConstantPtr?
        let res = ergo_lib_ergo_box_candidate_builder_register_value(
            self.pointer,
            registerId.rawValue,
            &ptr
        )
        assert(res.error == nil)
        if res.is_some {
            return Constant(withRawPointer: ptr!)
        } else {
            return nil
        }
    }
    
    /// Delete register value(make register empty) for the given register id (R4-R9)
    func deleteRegisterValue(
        registerId: NonMandatoryRegisterId
    ) -> ErgoBoxCandidateBuilder {
        ergo_lib_ergo_box_candidate_builder_delete_register_value(
            self.pointer,
            registerId.rawValue
        )
        return self
    }
    
    /// Mint token, as defined in https://github.com/ergoplatform/eips/blob/master/eip-0004.md
    /// - Parameters
    ///  - `token`: token id(box id of the first input box in transaction) and token amount,
    ///  - `tokenName`: token name (will be encoded in R4),
    ///  - `tokenDesc`: token description (will be encoded in R5),
    ///  - `numDecimals`: number of decimals (will be encoded in R6)
    func mintToken(
        token: Token,
        tokenName: String,
        tokenDesc: String,
        numDecimals: UInt
    ) -> ErgoBoxCandidateBuilder {
        let _ =
            tokenName.withCString{tokenNameCStr in
                tokenDesc.withCString{tokenDescCStr in
                    ergo_lib_ergo_box_candidate_builder_mint_token(
                        self.pointer,
                        token.pointer,
                        tokenNameCStr,
                        tokenDescCStr,
                        numDecimals
                    )
                }
            }
        return self
    }
    
    /// Add given token id and token amount
    func addToken(
        tokenId: TokenId,
        tokenAmount: TokenAmount
    ) -> ErgoBoxCandidateBuilder {
        ergo_lib_ergo_box_candidate_builder_add_token(
            self.pointer,
            tokenId.pointer,
            tokenAmount.pointer
        )
        return self
    }
    
    /// Build the box candidate
    func build() throws -> ErgoBoxCandidate {
        var ptr: ErgoBoxCandidatePtr?
        let error = ergo_lib_ergo_box_candidate_builder_build(self.pointer, &ptr)
        try checkError(error)
        return ErgoBoxCandidate(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_lib_ergo_box_candidate_builder_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC
import SwiftyJSON

class ErgoBoxCandidateBuilder {
    internal var pointer: BoxIdPtr
    
    init(boxValue: BoxValue, contract: Contract, creationHeight: UInt32) throws {
        var ptr: ErgoBoxCandidateBuilderPtr?
        let error = ergo_wallet_ergo_box_candidate_builder_new(
            boxValue.pointer,
            contract.pointer,
            creationHeight,
            &ptr
        )
        try checkError(error)
        self.pointer = ptr!
    }
    
    func setMinBoxValuePerByte(minBoxValuePerByte: UInt32) throws -> ErgoBoxCandidateBuilder {
        let error = ergo_wallet_ergo_box_candidate_builder_set_min_box_value_per_byte(
            self.pointer,
            minBoxValuePerByte
        )
        try checkError(error)
        return self
    }
    
    func getMinBoxValuePerByte() throws -> UInt32 {
        let res = ergo_wallet_ergo_box_candidate_builder_min_box_value_per_byte(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func setValue(boxValue: BoxValue) throws -> ErgoBoxCandidateBuilder {
        let error = ergo_wallet_ergo_box_candidate_builder_set_value(
            self.pointer,
            boxValue.pointer
        )
        try checkError(error)
        return self
    }
    
    func getValue() throws -> BoxValue {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_ergo_box_candidate_builder_value(self.pointer, &ptr)
        try checkError(error)
        return BoxValue(withPtr: ptr!)
    }
    
    func calcBoxSizeBytes() throws -> UInt {
        let res = ergo_wallet_ergo_box_candidate_builder_calc_box_size_bytes(self.pointer)
        try checkError(res.error)
        return res.value
    }
    
    func calcMinBoxValue() throws -> BoxValue {
        var ptr: BoxValuePtr?
        let error = ergo_wallet_ergo_box_candidate_calc_min_box_value(self.pointer, &ptr)
        try checkError(error)
        return BoxValue(withPtr: ptr!)
    }
    
    func setRegisterValue(
        registerId: NonMandatoryRegisterId,
        constant: Constant
    ) throws -> ErgoBoxCandidateBuilder {
        let error = ergo_wallet_ergo_box_candidate_builder_set_register_value(
            self.pointer,
            registerId.rawValue,
            constant.pointer
        )
        try checkError(error)
        return self
    }
    
    func getRegisterValue(registerId: NonMandatoryRegisterId) throws -> Constant? {
        var ptr: ConstantPtr?
        let res = ergo_wallet_ergo_box_candidate_builder_register_value(
            self.pointer,
            registerId.rawValue,
            &ptr
        )
        try checkError(res.error)
        if res.is_some {
            return Constant(withPtr: ptr!)
        } else {
            return nil
        }
    }
    
    func deleteRegisterValue(
        registerId: NonMandatoryRegisterId
    ) throws -> ErgoBoxCandidateBuilder {
        let error = ergo_wallet_ergo_box_candidate_builder_delete_register_value(
            self.pointer,
            registerId.rawValue
        )
        try checkError(error)
        return self
    }
    
    func mintToken(
        token: Token,
        tokenName: String,
        tokenDesc: String,
        numDecimals: UInt
    ) throws -> ErgoBoxCandidateBuilder {
        let error = tokenName.withCString{tokenNameCStr -> ErrorPtr? in
            let error = tokenDesc.withCString{tokenDescCStr in
                ergo_wallet_ergo_box_candidate_builder_mint_token(
                    self.pointer,
                    token.pointer,
                    tokenNameCStr,
                    tokenDescCStr,
                    numDecimals
                )
            }
            return error
        }
        
        try checkError(error)
        return self
    }
    
    func addToken(
        tokenId: TokenId,
        tokenAmount: TokenAmount
    ) throws -> ErgoBoxCandidateBuilder {
        let error = ergo_wallet_ergo_box_candidate_builder_add_token(
            self.pointer,
            tokenId.pointer,
            tokenAmount.pointer
        )
        try checkError(error)
        return self
    }
    
    func build() throws -> ErgoBoxCandidate {
        var ptr: ErgoBoxCandidatePtr?
        let error = ergo_wallet_ergo_box_candidate_builder_build(self.pointer, &ptr)
        try checkError(error)
        return ErgoBoxCandidate(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_wallet_ergo_box_candidate_builder_delete(self.pointer)
    }
}

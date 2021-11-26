import Foundation
import ErgoLibC
import SwiftyJSON

class ErgoBoxCandidateBuilder {
    internal var pointer: BoxIdPtr
    
    init(boxValue: BoxValue, contract: Contract, creationHeight: UInt32) {
        var ptr: ErgoBoxCandidateBuilderPtr?
        ergo_wallet_ergo_box_candidate_builder_new(
            boxValue.pointer,
            contract.pointer,
            creationHeight,
            &ptr
        )
        self.pointer = ptr!
    }
    
    func setMinBoxValuePerByte(minBoxValuePerByte: UInt32) -> ErgoBoxCandidateBuilder {
        ergo_wallet_ergo_box_candidate_builder_set_min_box_value_per_byte(
            self.pointer,
            minBoxValuePerByte
        )
        return self
    }
    
    func getMinBoxValuePerByte() -> UInt32 {
        ergo_wallet_ergo_box_candidate_builder_min_box_value_per_byte(self.pointer)
    }
    
    func setValue(boxValue: BoxValue) -> ErgoBoxCandidateBuilder {
        ergo_wallet_ergo_box_candidate_builder_set_value(
            self.pointer,
            boxValue.pointer
        )
        return self
    }
    
    func getValue() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_wallet_ergo_box_candidate_builder_value(self.pointer, &ptr)
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
    ) -> ErgoBoxCandidateBuilder {
        ergo_wallet_ergo_box_candidate_builder_set_register_value(
            self.pointer,
            registerId.rawValue,
            constant.pointer
        )
        return self
    }
    
    func getRegisterValue(registerId: NonMandatoryRegisterId) -> Constant? {
        var ptr: ConstantPtr?
        let res = ergo_wallet_ergo_box_candidate_builder_register_value(
            self.pointer,
            registerId.rawValue,
            &ptr
        )
        assert(res.error == nil)
        if res.is_some {
            return Constant(withPtr: ptr!)
        } else {
            return nil
        }
    }
    
    func deleteRegisterValue(
        registerId: NonMandatoryRegisterId
    ) -> ErgoBoxCandidateBuilder {
        ergo_wallet_ergo_box_candidate_builder_delete_register_value(
            self.pointer,
            registerId.rawValue
        )
        return self
    }
    
    func mintToken(
        token: Token,
        tokenName: String,
        tokenDesc: String,
        numDecimals: UInt
    ) -> ErgoBoxCandidateBuilder {
        let _ =
            tokenName.withCString{tokenNameCStr in
                tokenDesc.withCString{tokenDescCStr in
                    ergo_wallet_ergo_box_candidate_builder_mint_token(
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
    
    func addToken(
        tokenId: TokenId,
        tokenAmount: TokenAmount
    ) -> ErgoBoxCandidateBuilder {
        ergo_wallet_ergo_box_candidate_builder_add_token(
            self.pointer,
            tokenId.pointer,
            tokenAmount.pointer
        )
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

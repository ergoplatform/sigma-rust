import Foundation
import ErgoLibC

class TxBuilder {
    internal var pointer: TxBuilderPtr
    
    init(
        boxSelection : BoxSelection,
        outputCandidates: ErgoBoxCandidates,
        currentHeight: UInt32,
        feeAmount: BoxValue,
        changeAddress: Address,
        minChangeValue: BoxValue
    ) {
        var ptr: TxBuilderPtr?
        ergo_wallet_tx_builder_new(
            boxSelection.pointer,
            outputCandidates.pointer,
            currentHeight,
            feeAmount.pointer,
            changeAddress.pointer,
            minChangeValue.pointer,
            &ptr
        )
        self.pointer = ptr!
    }
    
    static func SUGGESTED_TX_FEE() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_wallet_tx_builder_suggested_tx_fee(&ptr)
        return BoxValue(withRawPointer: ptr!)
    }
    
    func setDataInputs(dataInputs: DataInputs) {
        ergo_wallet_tx_builder_set_data_inputs(self.pointer, dataInputs.pointer)
    }
    
    func getDataInputs() -> DataInputs {
        var ptr: DataInputsPtr?
        ergo_wallet_tx_builder_data_inputs(self.pointer, &ptr)
        return DataInputs(withPtr: ptr!)
    }
    
    func build() throws -> UnsignedTransaction {
        var ptr: UnsignedTransactionPtr?
        let error = ergo_wallet_tx_builder_build(self.pointer, &ptr)
        try checkError(error)
        return UnsignedTransaction(withRawPointer: ptr!)
    }
    
    func getBoxSelection() -> BoxSelection {
        var ptr: BoxSelectionPtr?
        ergo_wallet_tx_builder_box_selection(self.pointer, &ptr)
        return BoxSelection(withPtr: ptr!)
    }
    
    func getOutputCandidates() -> ErgoBoxCandidates {
        var ptr: ErgoBoxCandidatesPtr?
        ergo_wallet_tx_builder_output_candidates(self.pointer, &ptr)
        return ErgoBoxCandidates(withRawPointer: ptr!)
    }
    
    func getCurrentHeight() -> UInt32 {
        return ergo_wallet_tx_builder_current_height(self.pointer)
    }
    
    func getFeeAmount() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_wallet_tx_builder_fee_amount(self.pointer, &ptr)
        return BoxValue(withRawPointer: ptr!)
    }
    
    func getChangeAddress() -> Address {
        var ptr: AddressPtr?
        ergo_wallet_tx_builder_change_address(self.pointer, &ptr)
        return Address(withRawPointer: ptr!)
    }
    
    func getMinChangeValue() -> BoxValue {
        var ptr: BoxValuePtr?
        ergo_wallet_tx_builder_min_change_value(self.pointer, &ptr)
        return BoxValue(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_wallet_tx_builder_delete(self.pointer)
    }
}

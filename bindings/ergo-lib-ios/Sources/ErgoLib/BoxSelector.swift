import Foundation
import ErgoLibC

class BoxSelection {
    internal var pointer: BoxSelectionPtr
    
    init(ergoBoxes: ErgoBoxes, changeErgoBoxes: ErgoBoxAssetsDataList) throws {
        var ptr: BoxSelectionPtr?
        ergo_wallet_box_selection_new(ergoBoxes.pointer, changeErgoBoxes.pointer, &ptr)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    func getBoxes() -> ErgoBoxes {
        var ptr: ErgoBoxesPtr?
        ergo_wallet_box_selection_boxes(self.pointer, &ptr)
        return ErgoBoxes(withRawPointer: ptr!)
    }
    
    func getChangeBoxes() -> ErgoBoxAssetsDataList {
        var ptr: ErgoBoxAssetsDataListPtr?
        ergo_wallet_box_selection_change(self.pointer, &ptr)
        return ErgoBoxAssetsDataList(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_wallet_box_selection_delete(self.pointer)
    }
}

extension BoxSelection: Equatable {
    static func ==(lhs: BoxSelection, rhs: BoxSelection) -> Bool {
        ergo_wallet_box_selection_eq(lhs.pointer, rhs.pointer)
    }
}
class SimpleBoxSelector {
    internal var pointer: SimpleBoxSelectorPtr
    
    init() {
        var ptr: SimpleBoxSelectorPtr?
        ergo_wallet_simple_box_selector_new(&ptr)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    func select( inputs: ErgoBoxes,
          targetBalance: BoxValue,
          targetTokens: Tokens
    ) throws -> BoxSelection {
        var ptr: BoxSelectionPtr?
        ergo_wallet_simple_box_selector_select(
            self.pointer,
            inputs.pointer,
            targetBalance.pointer,
            targetTokens.pointer,
            &ptr
        )
        return BoxSelection(withPtr: ptr!)
    }
    
    deinit {
        ergo_wallet_simple_box_selector_delete(self.pointer)
    }
}

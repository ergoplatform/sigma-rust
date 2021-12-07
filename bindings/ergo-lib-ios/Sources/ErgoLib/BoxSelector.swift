import Foundation
import ErgoLibC

/// Selected boxes with change boxes. Instances of this class are created by ``BoxSelector``.
class BoxSelection {
    internal var pointer: BoxSelectionPtr
    
    /// Create a selection to easily inject custom selection algorithms
    init(ergoBoxes: ErgoBoxes, changeErgoBoxes: ErgoBoxAssetsDataList) {
        var ptr: BoxSelectionPtr?
        ergo_lib_box_selection_new(ergoBoxes.pointer, changeErgoBoxes.pointer, &ptr)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``BoxSelectionPtr``. Note: we must ensure that no other instance
    /// of ``BoxSelection`` can hold this pointer.
    internal init(withRawPointer ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    /// Selected boxes to spend as transaction inputs
    func getBoxes() -> ErgoBoxes {
        var ptr: ErgoBoxesPtr?
        ergo_lib_box_selection_boxes(self.pointer, &ptr)
        return ErgoBoxes(withRawPointer: ptr!)
    }
    
    /// Selected boxes to use as change
    func getChangeBoxes() -> ErgoBoxAssetsDataList {
        var ptr: ErgoBoxAssetsDataListPtr?
        ergo_lib_box_selection_change(self.pointer, &ptr)
        return ErgoBoxAssetsDataList(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_lib_box_selection_delete(self.pointer)
    }
}

extension BoxSelection: Equatable {
    static func ==(lhs: BoxSelection, rhs: BoxSelection) -> Bool {
        ergo_lib_box_selection_eq(lhs.pointer, rhs.pointer)
    }
}

/// Naive box selector, collects inputs until target balance is reached
class SimpleBoxSelector {
    internal var pointer: SimpleBoxSelectorPtr
    
    init() {
        var ptr: SimpleBoxSelectorPtr?
        ergo_lib_simple_box_selector_new(&ptr)
        self.pointer = ptr!
    }
    
    internal init(withPtr ptr: BoxIdPtr) {
        self.pointer = ptr
    }
    
    /// Selects inputs to satisfy target balance and tokens.
    /// - Parameters:
    ///  - `inputs`: available inputs (returns an error, if empty),
    ///  - `targetBalance: coins (in nanoERGs) needed,
    ///  - `targetTokens: amount of tokens needed.
    /// - Returns: selected inputs and box assets(value+tokens) with change.
    func select( inputs: ErgoBoxes,
          targetBalance: BoxValue,
          targetTokens: Tokens
    ) throws -> BoxSelection {
        var ptr: BoxSelectionPtr?
        let error = ergo_lib_simple_box_selector_select(
            self.pointer,
            inputs.pointer,
            targetBalance.pointer,
            targetTokens.pointer,
            &ptr
        )
        try checkError(error)
        return BoxSelection(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_lib_simple_box_selector_delete(self.pointer)
    }
}

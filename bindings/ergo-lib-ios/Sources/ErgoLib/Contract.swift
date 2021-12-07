import Foundation
import ErgoLibC

/// Defines the contract(script) that will be guarding box contents
class Contract {
    internal var pointer: ContractPtr
    
    internal init(withRawPtr ptr: ContractPtr) {
        self.pointer = ptr
    }
    
    /// Create new contract from ErgoTree
    init(fromErgoTree: ErgoTree) {
        var ptr:  ContractPtr?
        ergo_lib_contract_new(fromErgoTree.pointer, &ptr)
        self.pointer = ptr!
    }
    
    /// Compiles a contract from ErgoScript source code
    init(compileFromString: String) throws {
        var ptr: ContractPtr?
        let error = compileFromString.withCString { cs in
            ergo_lib_contract_compile(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Create new contract that allow spending of the guarded box by a given recipient ([`Address`])
    init(payToAddress: Address) throws {
        var ptr: ContractPtr?
        let error = ergo_lib_contract_pay_to_address(payToAddress.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Get the ErgoTree of the contract
    func getErgoTree() -> ErgoTree {
        var boxIdPtr: ErgoTreePtr?
        ergo_lib_contract_ergo_tree(self.pointer, &boxIdPtr)
        return ErgoTree(withRawPointer: boxIdPtr!)
    }
        
    deinit {
        ergo_lib_contract_delete(self.pointer)
    }
}

extension Contract: Equatable {
    static func ==(lhs: Contract, rhs: Contract) -> Bool {
        ergo_lib_contract_eq(lhs.pointer, rhs.pointer)
    }
}

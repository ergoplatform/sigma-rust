import Foundation
import ErgoLibC

class Wallet {
    internal var pointer: WalletPtr
    
    init(mnemonicPhrase : String, mnemonicPass: String) throws {
        var ptr: WalletPtr?
        let error =
            mnemonicPhrase.withCString{phrase -> ErrorPtr? in
                mnemonicPass.withCString{pass in
                    ergo_wallet_wallet_from_mnemonic(phrase, pass, &ptr)
                }
            }
        try checkError(error)
        self.pointer = ptr!
    }
    
    init(secrets: SecretKeys) {
        var ptr: WalletPtr?
        ergo_wallet_wallet_from_secrets(secrets.pointer, &ptr)
        self.pointer = ptr!
    }
    
    func signTransaction(
        stateContext: ErgoStateContext,
        unsignedTx: UnsignedTransaction,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes
    ) throws -> Transaction {
        var ptr: TransactionPtr?
        let error = ergo_wallet_wallet_sign_transaction(self.pointer, stateContext.pointer, unsignedTx.pointer, boxesToSpend.pointer, dataBoxes.pointer, &ptr)
        try checkError(error)
        return Transaction(withRawPointer: ptr!)
    }
    
    func signReducedTransaction(
        reducedTx: ReducedTransaction
    ) throws -> Transaction {
        var ptr: TransactionPtr?
        let error = ergo_wallet_wallet_sign_reduced_transaction(self.pointer, reducedTx.pointer, &ptr)
        try checkError(error)
        return Transaction(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_wallet_wallet_delete(self.pointer)
    }
}

import Foundation
import ErgoLibC

class MnemonicGenerator {
    internal var pointer: MnemonicGeneratorPtr

    /// Create new ``MnemonicGenerator`` instance
    init(language: String, strength: UInt32) throws {
        var ptr: MnemonicGeneratorPtr?
        let error = language.withCString { lang in
            ergo_lib_mnemonic_generator(lang, strength, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }

    /// Generate mnemonic sentence using random entropy
    func generate() throws -> String {
        let res = ergo_lib_mnemonic_generator_generate(self.pointer)
        try checkError(res.error)
        let mnemonic = String(cString: res.value)
        ergo_lib_mnemonic_generator_free_mnemonic(res.value)
        return mnemonic
    }

    /// Generate mnemonic sentence using provided entropy
    func generateFromEntropy(entropy: [UInt8]) throws -> String {
        let pointer = UnsafeMutablePointer<UInt8>.allocate(capacity: entropy.count)
        pointer.initialize(from: entropy, count: entropy.count)
        defer {
            pointer.deinitialize(count: entropy.count)
            pointer.deallocate()
        }
        let res = ergo_lib_mnemonic_generator_generate_from_entropy(
            self.pointer, pointer, UInt(entropy.count))
        try checkError(res.error)
        let mnemonic = String(cString: res.value)
        ergo_lib_mnemonic_generator_free_mnemonic(res.value)
        return mnemonic
    }
}

class Wallet {
    internal var pointer: WalletPtr
    
    /// Create ``Wallet`` instance loading secret key from mnemonic. Throws error if a DlogSecretKey cannot be
    /// parsed from the provided phrase
    init(mnemonicPhrase : String, mnemonicPass: String) throws {
        var ptr: WalletPtr?
        let error =
            mnemonicPhrase.withCString{phrase -> ErrorPtr? in
                mnemonicPass.withCString{pass in
                    ergo_lib_wallet_from_mnemonic(phrase, pass, &ptr)
                }
            }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Create ``Wallet`` from secrets
    init(secrets: SecretKeys) {
        var ptr: WalletPtr?
        ergo_lib_wallet_from_secrets(secrets.pointer, &ptr)
        self.pointer = ptr!
    }

    /// Add a secret to the wallets prover
    func addSecret(secret: SecretKey) throws {
        let error = ergo_lib_wallet_add_secret(self.pointer, secret.pointer)
        try checkError(error)
    }
    
    /// Sign a transaction
    func signTransaction(
        stateContext: ErgoStateContext,
        unsignedTx: UnsignedTransaction,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes
    ) throws -> Transaction {
        var ptr: TransactionPtr?
        let error = ergo_lib_wallet_sign_transaction(self.pointer, stateContext.pointer, unsignedTx.pointer, boxesToSpend.pointer, dataBoxes.pointer, &ptr)
        try checkError(error)
        return Transaction(withRawPointer: ptr!)
    }
    
    /// Sign a multi signature transaction
    func signTransactionMulti(
        stateContext: ErgoStateContext,
        unsignedTx: UnsignedTransaction,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes,
        txHints: TransactionHintsBag
    ) throws -> Transaction {
        var ptr: TransactionPtr?
        let error = ergo_lib_wallet_sign_transaction_multi(
            self.pointer,
            stateContext.pointer,
            unsignedTx.pointer,
            boxesToSpend.pointer,
            dataBoxes.pointer,
            txHints.pointer,
            &ptr
        )
        try checkError(error)
        return Transaction(withRawPointer: ptr!)
    }
    
    /// Signs a reduced transaction (generating proofs for inputs)
    func signReducedTransaction(
        reducedTx: ReducedTransaction
    ) throws -> Transaction {
        var ptr: TransactionPtr?
        let error = ergo_lib_wallet_sign_reduced_transaction(self.pointer, reducedTx.pointer, &ptr)
        try checkError(error)
        return Transaction(withRawPointer: ptr!)
    }
    
    /// Signs a multi signature reduced transaction
    func signReducedTransactionMulti(
        reducedTx: ReducedTransaction,
        txHints: TransactionHintsBag
    ) throws -> Transaction {
        var ptr: TransactionPtr?
        let error = ergo_lib_wallet_sign_reduced_transaction_multi(
            self.pointer,
            reducedTx.pointer,
            txHints.pointer,
            &ptr
        )
        try checkError(error)
        return Transaction(withRawPointer: ptr!)
    }
    
    /// Generate Commitments for unsigned tx
    func generateCommitments(
        stateContext: ErgoStateContext,
        unsignedTx: UnsignedTransaction,
        boxesToSpend: ErgoBoxes,
        dataBoxes: ErgoBoxes
    ) throws -> TransactionHintsBag {
        var ptr: TransactionHintsBagPtr?
        let error = ergo_lib_wallet_generate_commitments(
            self.pointer,
            stateContext.pointer,
            unsignedTx.pointer,
            boxesToSpend.pointer,
            dataBoxes.pointer,
            &ptr
        )
        try checkError(error)
        return TransactionHintsBag(withRawPointer: ptr!)
    }
    
    /// Generate Commitments for reduced transaction
    func generateCommitmentsForReducedTransaction(
        reducedTx: ReducedTransaction
    ) throws -> TransactionHintsBag {
        var ptr: TransactionHintsBagPtr?
        let error = ergo_lib_wallet_generate_commitments_for_reduced_transaction(
            self.pointer,
            reducedTx.pointer,
            &ptr
        )
        try checkError(error)
        return TransactionHintsBag(withRawPointer: ptr!)
    }
    
    /// Sign an arbitrary message using a P2PK address
    func signedMessageUsingP2PK(
        address: Address,
        message: [UInt8]
    ) throws -> SignedMessage {
        var ptr: SignedMessagePtr?
        let error = ergo_lib_wallet_sign_message_using_p2pk(
            self.pointer,
            address.pointer,
            message,
            UInt(message.count),
            &ptr
        )
        try checkError(error)
        return SignedMessage(withRawPointer: ptr!)
    }
    
    deinit {
        ergo_lib_wallet_delete(self.pointer)
    }
}

class SignedMessage {
    internal var pointer: SignedMessagePtr
    
    /// Takes ownership of an existing ``SignedMessagePtr``. Note: we must ensure that no other instance
    /// of ``SignedMessage`` can hold this pointer.
    internal init(withRawPointer ptr: SignedMessagePtr) {
        self.pointer = ptr
    }
    
    deinit {
        ergo_lib_signed_message_delete(self.pointer)
    }
}

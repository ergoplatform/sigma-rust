import Foundation
import ErgoWalletC

// TODO: extract into files

enum WalletError: Error {
    case walletCError(reason: String)
}

class UnspentBoxes {
    internal var pointer: UnspentBoxesPtr

    init(withJson json: String) throws {
        self.pointer = try UnspentBoxes.from_json(json: json)
    }

    private static func from_json(json: String) throws -> UnspentBoxesPtr {
        var unspentBoxesPtr: UnspentBoxesPtr?
        let error = json.withCString { cs in
            ergo_wallet_unspent_boxes_from_json(cs, &unspentBoxesPtr)
        }
        try checkError(error)
        return unspentBoxesPtr!
    }

    deinit {
        ergo_wallet_unspent_boxes_delete(self.pointer)
    }
}

private func checkError(_ error: ErrorPtr?) throws {
    if error == nil {
        return
    }

    let cStringReason = ergo_wallet_error_to_string(error)
    let reason = String(cString: cStringReason!)
    ergo_wallet_delete_string(cStringReason)
    ergo_wallet_delete_error(error)
    throw WalletError.walletCError(reason: reason)
}

class Transaction {
    private var pointer: TransactionPtr

    internal init(withRawPointer pointer: TransactionPtr) {
        self.pointer = pointer
    }

    func toJson() throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_signed_tx_to_json(self.pointer, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }

    deinit {
        ergo_wallet_delete_signed_tx(self.pointer)
    }
}

class ErgoStateContext {
    internal var pointer: ErgoStateContextPtr

    init(withJson json: String) throws {
        self.pointer = try ErgoStateContext.fromJson(json: json)
    }

    private static func fromJson(json: String) throws -> ErgoStateContextPtr {
        var ergoStateContextPtr: ErgoStateContextPtr?
        let error = json.withCString { cs in
            ergo_wallet_ergo_state_context_from_json(cs, &ergoStateContextPtr)
        }
        try checkError(error)
        return ergoStateContextPtr!
    }

    deinit {
        ergo_wallet_ergo_state_context_delete(self.pointer)
    }
}

class Address {
    internal var pointer: AddressPtr

    init(withTestnetAddress addressStr: String) throws {
        self.pointer = try Address.fromTestnetAddress(addressStr: addressStr)
    }

    private static func fromTestnetAddress(addressStr: String) throws -> AddressPtr {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_wallet_address_from_testnet(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_address_delete(self.pointer)
    }
}

class ErgoBoxCandidate {
    internal var pointer: ErgoBoxCandidatePtr

    internal init(withRawPointer pointer: ErgoBoxCandidatePtr) {
        self.pointer = pointer
    }

    static func payToAddress(recipient: Address,
                             value: UInt64, creationHeight: UInt32) throws -> ErgoBoxCandidate {
        var ergoBoxCandidatePtr: ErgoBoxCandidatePtr?
        let error = ergo_wallet_ergo_box_candidate_new_pay_to_address(recipient.pointer, 
                                                                     value,
                                                                     creationHeight,
                                                                     &ergoBoxCandidatePtr)
        try checkError(error)
        return ErgoBoxCandidate(withRawPointer: ergoBoxCandidatePtr!)
    }

    deinit {
        ergo_wallet_ergo_box_candidate_delete(self.pointer)
    }
}

class OutputBoxes {
    internal var pointer: OutputBoxesPtr

    init(box: ErgoBoxCandidate) throws {
        var ptr: OutputBoxesPtr?
        let error = ergo_wallet_output_boxes_new(box.pointer, &ptr)
        try checkError(error)
        self.pointer = ptr!
    }

    deinit {
        ergo_wallet_output_boxes_delete(self.pointer)
    }
}

class Wallet {
    internal var pointer: WalletPtr

    init(withMnemonic mnemonicPhrase: String) throws {
        var ptr: WalletPtr?
        let error = mnemonicPhrase.withCString { mnemonicPhraseC in
            ergo_wallet_wallet_from_mnemonic(mnemonicPhraseC, nil, 0, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }

    func new_signed_tx(ergoStateContext: ErgoStateContext, 
                              unspentBoxes: UnspentBoxes, 
                              outputBoxes: OutputBoxes,
                              sendChangeTo: Address, 
                              minChangeValue: UInt64,
                              txFeeAmount: UInt64) throws -> Transaction {
        var transactionPtr: TransactionPtr?
        let error = ergo_wallet_wallet_new_signed_tx(self.pointer, 
                                              ergoStateContext.pointer,
                                              unspentBoxes.pointer, 
                                              nil, // data input boxes
                                              outputBoxes.pointer,
                                              sendChangeTo.pointer,
                                              minChangeValue, 
                                              txFeeAmount, 
                                              &transactionPtr)
        try checkError(error)
        return Transaction(withRawPointer: transactionPtr!)
    }
}

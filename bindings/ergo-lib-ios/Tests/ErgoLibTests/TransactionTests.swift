
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class TransactionTests: XCTestCase {
    func testTxId() throws {
        let str = "93d344aa527e18e5a221db060ea1a868f46b61e4537e6e5f69ecc40334c15e38"
        let txId = try TxId(withString: str)
        XCTAssertEqual(try txId.toString(), str)
    }
    
    func testTxBuilder() throws {
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let jsonStr = """
        {
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }
        """
        let json = [jsonStr.data(using: .utf8, allowLossyConversion: false)!]
        let unspentBoxes = try ErgoBoxes(fromJSON: json)
        let contract = try Contract(payToAddress: recipient)
        let outboxValue = BoxValue.SAFE_USER_MIN()
        let outbox = try ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0)).build()
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let dataInputs = DataInputs()
        let boxSelector = SimpleBoxSelector()
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        let txBuilder = TxBuilder(
            boxSelection: boxSelection,
            outputCandidates: txOutputs,
            currentHeight: 0,
            feeAmount: fee,
            changeAddress: changeAddress
        )
        txBuilder.setDataInputs(dataInputs: dataInputs)
        let tx = try txBuilder.build()
        XCTAssertNoThrow(try tx.toJsonEIP12())
    }
    
    func testSignTx() throws {
        let sk = SecretKey()
        let inputContract = try Contract(payToAddress: sk.getAddress())
        let str = "93d344aa527e18e5a221db060ea1a868f46b61e4537e6e5f69ecc40334c15e38"
        let txId = try TxId(withString: str)
        let inputBox = try ErgoBox(boxValue: BoxValue(fromInt64: Int64(1000000000)), creationHeight: 0, contract: inputContract, txId: txId, index: 0, tokens: Tokens())
        
        // Create transaction that spends the 'simulated' box
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let unspentBoxes = ErgoBoxes()
        unspentBoxes.add(ergoBox: inputBox)
        let contract = try Contract(payToAddress: recipient)
        let outboxValue = BoxValue.SAFE_USER_MIN()
        let outbox = try ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0)).build()
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let dataInputs = DataInputs()
        let boxSelector = SimpleBoxSelector()
        // Note that swift forbids integer overflow/underflow by default with hard crash
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        let txBuilder = TxBuilder(boxSelection: boxSelection, outputCandidates: txOutputs, currentHeight: 0, feeAmount: fee, changeAddress: changeAddress)
        txBuilder.setDataInputs(dataInputs: dataInputs)
        let tx = try txBuilder.build()
        XCTAssertNoThrow(try tx.toJsonEIP12())
        
        let txDataInputs = try ErgoBoxes(fromJSON: [])
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let preHeader = PreHeader(withBlockHeader: blockHeaders.get(index: UInt(0))!)
        let ctx = try ErgoStateContext(preHeader: preHeader, headers: blockHeaders, parameters: Parameters())
        let secretKeys = SecretKeys()
        secretKeys.add(secretKey: sk)
        let wallet = Wallet(secrets: secretKeys)
        let signedTx = try wallet.signTransaction(stateContext: ctx, unsignedTx: tx, boxesToSpend: unspentBoxes, dataBoxes: txDataInputs)
        XCTAssertNoThrow(try signedTx.toJsonEIP12())
    }
    
    func testMintToken() throws {
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let jsonStr = """
        {
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }
        """
        let json = [jsonStr.data(using: .utf8, allowLossyConversion: false)!]
        let unspentBoxes = try ErgoBoxes(fromJSON: json)
        let contract = try Contract(payToAddress: recipient)
        let outboxValue = BoxValue.SAFE_USER_MIN()
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        
        let boxSelector = SimpleBoxSelector()
        // Note that swift forbids integer overflow/underflow by default with hard crash
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        
        // Mint token
        let tokenId = TokenId(fromBoxId: boxSelection.getBoxes().get(index: UInt(0))!.getBoxId())
        let token = Token(tokenId: tokenId, tokenAmount: try TokenAmount(fromInt64: Int64(1)))
        let boxBuilder = ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0))
            .mintToken(token: token, tokenName: "TKN", tokenDesc: "token desc", numDecimals: UInt(2))
        let outbox = try boxBuilder.build()
        
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let dataInputs = DataInputs()
        let txBuilder = TxBuilder(
            boxSelection: boxSelection,
            outputCandidates: txOutputs,
            currentHeight: 0,
            feeAmount: fee,
            changeAddress: changeAddress
        )
        txBuilder.setDataInputs(dataInputs: dataInputs)
        let tx = try txBuilder.build()
        XCTAssertNoThrow(try tx.toJsonEIP12())
    }
    
    func testBurnToken() throws {
        let jsonStr =
            """
                {
                  "boxId": "0cf7b9e71961cc473242de389c8e594a4e5d630ddd2e4e590083fb0afb386341",
                  "value": 11491500000,
                  "ergoTree": "100f040005c801056404000e2019719268d230fd9093e4db0e2e42a07883ffe976e77c7419efc1bb218a05d4ba04000500043c040204c096b10204020101040205c096b1020400d805d601b2a5730000d602e4c6a70405d6039c9d720273017302d604b5db6501fed9010463ededed93e4c67204050ec5a7938cb2db6308720473030001730492e4c672040605997202720390e4c6720406059a72027203d605b17204ea02d1edededededed93cbc27201e4c6a7060e917205730593db63087201db6308a793e4c6720104059db072047306d9010641639a8c720601e4c68c72060206057e72050593e4c6720105049ae4c6a70504730792c1720199c1a77e9c9a720573087309058cb072048602730a730bd901063c400163d802d6088c720601d6098c72080186029a7209730ceded8c72080293c2b2a5720900d0cde4c68c720602040792c1b2a5720900730d02b2ad7204d9010663cde4c672060407730e00",
                  "assets": [
                    {
                      "tokenId": "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac",
                      "amount": 1
                    }
                  ],
                  "creationHeight": 348198,
                  "additionalRegisters": {
                    "R4": "059acd9109",
                    "R5": "04f2c02a",
                    "R6": "0e20277c78751ff6f68d4dcd082eeea9506324911a875b6b9cd4d177d4fcab061327"
                  },
                  "transactionId": "5ed0e572a8c097b053965519a696f413f7be02754345e8ed650377e29a6dedb3",
                  "index": 0
                }
            """
        let json = [jsonStr.data(using: .utf8, allowLossyConversion: false)!]
        let unspentBoxes = try ErgoBoxes(fromJSON: json)
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let tokenId = try TokenId(fromBase16EncodedString: "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac")
        let token = Token(tokenId: tokenId, tokenAmount: try TokenAmount(fromInt64: Int64(1)))
        let boxSelector = SimpleBoxSelector()
        let tokens = Tokens()
        try tokens.add(token: token)
        let outboxValue = BoxValue.SAFE_USER_MIN()
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        
        // Note that swift forbids integer overflow/underflow by default with hard crash
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        
        // Select tokens from inputs
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: tokens)
        let contract = try Contract(payToAddress: recipient)
        // but don't put selected tokens in the output box (burn them)
        let boxBuilder = ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0))
        let outbox = try boxBuilder.build()
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let dataInputs = DataInputs()
        let txBuilder = TxBuilder(boxSelection: boxSelection, outputCandidates: txOutputs, currentHeight: UInt32(0), feeAmount: fee, changeAddress: changeAddress)
        txBuilder.setDataInputs(dataInputs: dataInputs)
        txBuilder.setTokenBurnPermit(tokens: tokens)
        XCTAssertNoThrow(try txBuilder.build())
    }
    
    func testUsingSignedTxAsInputInNewTx() throws {
        let sk = SecretKey()
        let inputContract = try Contract(payToAddress: sk.getAddress())
        let str = "0000000000000000000000000000000000000000000000000000000000000000"
        let txId = try TxId(withString: str)
        let inputBox = try ErgoBox(boxValue: BoxValue(fromInt64: Int64(100000000000)), creationHeight: 0, contract: inputContract, txId: txId, index: 0, tokens: Tokens())
        
        // Create transaction that spends the 'simulated' box
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let unspentBoxes = ErgoBoxes()
        unspentBoxes.add(ergoBox: inputBox)
        let contract = try Contract(payToAddress: recipient)
        let outboxValue = try BoxValue(fromInt64: Int64(10000000000))
        let outbox = try ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0)).build()
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let dataInputs = DataInputs()
        let boxSelector = SimpleBoxSelector()
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        let txBuilder = TxBuilder(boxSelection: boxSelection, outputCandidates: txOutputs, currentHeight: 0, feeAmount: fee, changeAddress: changeAddress)
        txBuilder.setDataInputs(dataInputs: dataInputs)
        let tx = try txBuilder.build()
        XCTAssertNoThrow(try tx.toJsonEIP12())
        
        let txDataInputs = try ErgoBoxes(fromJSON: [])
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let preHeader = PreHeader(withBlockHeader: blockHeaders.get(index: UInt(0))!)
        let ctx = try ErgoStateContext(preHeader: preHeader, headers: blockHeaders, parameters: Parameters())
        let secretKeys = SecretKeys()
        secretKeys.add(secretKey: sk)
        let wallet = Wallet(secrets: secretKeys)
        let signedTx = try wallet.signTransaction(stateContext: ctx, unsignedTx: tx, boxesToSpend: unspentBoxes, dataBoxes: txDataInputs)
        XCTAssertEqual(signedTx.getOutputs().get(index: UInt(0))!.getBoxValue().toInt64(), Int64(10000000000))
        XCTAssertNoThrow(try signedTx.toJsonEIP12())
        
        // New tx
        let newOutboxValue = try BoxValue(fromInt64: Int64(1000000000))
        let newOutbox = try ErgoBoxCandidateBuilder(boxValue: newOutboxValue, contract: contract, creationHeight: UInt32(0)).build()
        let newTxOutputs = ErgoBoxCandidates()
        newTxOutputs.add(ergoBoxCandidate: newOutbox)
        let newBoxSelector = SimpleBoxSelector()
        let newTargetBalance = try BoxValue.sumOf(boxValue0: newOutboxValue, boxValue1: fee)
        let newBoxSelection = try newBoxSelector.select(
            inputs: signedTx.getOutputs(),
            targetBalance: newTargetBalance,
            targetTokens: Tokens()
        )
        let newTxBuilder = TxBuilder(
            boxSelection: newBoxSelection,
            outputCandidates: newTxOutputs,
            currentHeight: UInt32(0),
            feeAmount: fee,
            changeAddress: changeAddress
        )
        XCTAssertNoThrow(try newTxBuilder.build())
    }
    
    func testTxFromUnsignedTx() throws {
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let jsonStr = """
        {
          "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
          "value": 67500000000,
          "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
          "assets": [],
          "creationHeight": 284761,
          "additionalRegisters": {},
          "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
          "index": 1
        }
        """
        let json = [jsonStr.data(using: .utf8, allowLossyConversion: false)!]
        let unspentBoxes = try ErgoBoxes(fromJSON: json)
        let contract = try Contract(payToAddress: recipient)
        let outboxValue = BoxValue.SAFE_USER_MIN()
        let outbox = try ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0)).build()
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let dataInputs = DataInputs()
        let boxSelector = SimpleBoxSelector()
        // Note that swift forbids integer overflow/underflow by default with hard crash
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        let txBuilder = TxBuilder(
            boxSelection: boxSelection,
            outputCandidates: txOutputs,
            currentHeight: 0,
            feeAmount: fee,
            changeAddress: changeAddress
        )
        txBuilder.setDataInputs(dataInputs: dataInputs)
        let tx = try txBuilder.build()
        let bytes = [1, 1, 2, 255].map{ UInt8($0) }
        let proof = try ByteArray(fromBytes: bytes)
        let proofs = ByteArrays()
        proofs.add(byteArray: proof)
        let signedTx = try Transaction.init(unsignedTx: tx, proofs: proofs)
        XCTAssertEqual(signedTx.getInputs().get(index: UInt(0))!.getSpendingProof().toBytes(), bytes)
    }
    
    func testWalletMnemonic() throws {
        let phrase = "change me do not use me change me do not use me"
        let pass = "password1234"
        XCTAssertNoThrow(try Wallet(mnemonicPhrase: phrase, mnemonicPass: pass))
    }
    
    func testMultiSigTx() throws {
        let aliceSecret =
            try SecretKey(fromBytes: base16StringToBytes("e726ad60a073a49f7851f4d11a83de6c9c7f99e17314fcce560f00a51a8a3d18")!
        )
        let bobSecret =
            try SecretKey(fromBytes: base16StringToBytes("9e6616b4e44818d21b8dfdd5ea87eb822480e7856ab910d00f5834dc64db79b3")!)
        let alicePkBytes = base16StringToBytes("cd03c8e1527efae4be9868cea6767157fcccac66489842738efed0a302e4f81710d0")!
        
        // Pay 2 Script address of a multi_sig contract with contract { alicePK && bobPK }
        let multiSigAddress =
          try Address(withTestnetAddress: "JryiCXrc7x5D8AhS9DYX1TDzW5C5mT6QyTMQaptF76EQkM15cetxtYKq3u6LymLZLVCyjtgbTKFcfuuX9LLi49Ec5m2p6cwsg5NyEsCQ7na83yEPN")
        let inputContract = try Contract.init(payToAddress: multiSigAddress)
        let txId = try TxId(withString: "0000000000000000000000000000000000000000000000000000000000000000")
        let inputBox = try ErgoBox(
            boxValue: try BoxValue(fromInt64: 1000000000),
            creationHeight: UInt32(0),
            contract: inputContract,
            txId: txId,
            index: UInt16(0),
            tokens: Tokens()
        )
        
        // create a transaction that spends the "simulated" box
        let recipient = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let unspentBoxes = ErgoBoxes()
        unspentBoxes.add(ergoBox: inputBox)
        let contract = try Contract(payToAddress: recipient)
        let outboxValue = BoxValue.SAFE_USER_MIN()
        let outbox = try ErgoBoxCandidateBuilder(boxValue: outboxValue, contract: contract, creationHeight: UInt32(0)).build()
        let txOutputs = ErgoBoxCandidates()
        txOutputs.add(ergoBoxCandidate: outbox)
        let fee = TxBuilder.SUGGESTED_TX_FEE()
        let changeAddress = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let minChangeValue = BoxValue.SAFE_USER_MIN()
        let boxSelector = SimpleBoxSelector()
        let targetBalance = try BoxValue.sumOf(boxValue0: outboxValue, boxValue1: fee)
        let boxSelection = try boxSelector.select(
            inputs: unspentBoxes,
            targetBalance: targetBalance,
            targetTokens: Tokens()
        )
        let txBuilder = TxBuilder(
            boxSelection: boxSelection,
            outputCandidates: txOutputs,
            currentHeight: UInt32(0),
            feeAmount: fee,
            changeAddress: changeAddress
        )
        let tx = try txBuilder.build()
        let txDataInputs = try ErgoBoxes.init(fromJSON: [])
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let preHeader = PreHeader(withBlockHeader: blockHeaders.get(index: UInt(0))!)
        let ctx = try ErgoStateContext(preHeader: preHeader, headers: blockHeaders, parameters: Parameters())
        let sksAlice = SecretKeys()
        sksAlice.add(secretKey: aliceSecret)
        let walletAlice = Wallet(secrets: sksAlice)
        let sksBob = SecretKeys()
        sksBob.add(secretKey: bobSecret)
        let walletBob = Wallet(secrets: sksBob)
        let bobHints = try walletBob.generateCommitments(
            stateContext: ctx,
            unsignedTx: tx,
            boxesToSpend: unspentBoxes,
            dataBoxes: txDataInputs
        ).allHintsForInput(index: UInt(0))
        let bobKnown = bobHints.getCommitmentHint(index: UInt(0))!
        let bobOwn = bobHints.getCommitmentHint(index: UInt(1))!
        let hintsBag = HintsBag()
        hintsBag.addCommitmentHint(hint: bobKnown)
        let aliceTxHintsBag = TransactionHintsBag()
        aliceTxHintsBag.addHintsForInput(index: UInt(0), hintsBag: hintsBag)
        let partialSigned = try walletAlice.signTransactionMulti(
            stateContext: ctx,
            unsignedTx: tx,
            boxesToSpend: unspentBoxes,
            dataBoxes: txDataInputs,
            txHints: aliceTxHintsBag
        )
        let realPropositions = Propositions()
        let simulatedPropositions = Propositions()
        try realPropositions.addProposition(fromBytes: alicePkBytes)
        let bobHintsBag = try extractHintsFromSignedTransaction(
            transaction: partialSigned,
            stateContext: ctx,
            boxesToSpend: unspentBoxes,
            dataBoxes: txDataInputs,
            realPropositions: realPropositions,
            simulatedPropositions: simulatedPropositions
        ).allHintsForInput(index: UInt(0))
        bobHintsBag.addCommitmentHint(hint: bobOwn)
        let bobTxHintsBag = TransactionHintsBag()
        bobTxHintsBag.addHintsForInput(index: UInt(0), hintsBag: bobHintsBag)
        XCTAssertNoThrow(
            try walletBob.signTransactionMulti(
                stateContext: ctx,
                unsignedTx: tx,
                boxesToSpend: unspentBoxes,
                dataBoxes: txDataInputs,
                txHints: bobTxHintsBag
            )
        )
        
    }
}

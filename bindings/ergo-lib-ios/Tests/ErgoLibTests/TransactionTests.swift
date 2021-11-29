
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
        // Note that swift forbids integer overflow/underflow by default with hard crash
        let targetBalance = try BoxValue(fromInt64: outboxValue.toInt64() + fee.toInt64())
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        let txBuilder = TxBuilder(boxSelection: boxSelection, outputCandidates: txOutputs, currentHeight: 0, feeAmount: fee, changeAddress: changeAddress, minChangeValue: minChangeValue)
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
        let targetBalance = try BoxValue(fromInt64: outboxValue.toInt64() + fee.toInt64())
        let boxSelection = try boxSelector.select(inputs: unspentBoxes, targetBalance: targetBalance, targetTokens: Tokens())
        let txBuilder = TxBuilder(boxSelection: boxSelection, outputCandidates: txOutputs, currentHeight: 0, feeAmount: fee, changeAddress: changeAddress, minChangeValue: minChangeValue)
        txBuilder.setDataInputs(dataInputs: dataInputs)
        let tx = try txBuilder.build()
        XCTAssertNoThrow(try tx.toJsonEIP12())
        
        let txDataInputs = try ErgoBoxes(fromJSON: [])
        let blockHeaders = try HeaderTests.generateBlockHeadersFromJSON()
        let preHeader = PreHeader(withBlockHeader: blockHeaders.get(index: UInt(0))!)
        let ctx = try ErgoStateContext(preHeader: preHeader, headers: blockHeaders)
        let secretKeys = SecretKeys()
        secretKeys.add(secretKey: sk)
    }
}

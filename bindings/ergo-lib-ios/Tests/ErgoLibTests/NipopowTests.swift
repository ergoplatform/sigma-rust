import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class PoPowHeaderTests: XCTestCase {
    // Test deserialization for PoPowHeader by comparing it with manually deserializing the header and interlinks
    func testFromJson() throws {
        // sample PoPowHeader generated from ergo-chain-generation testing infrastructure
        let json = """
          {
          "header": {
          "version": 1,
          "id": "fa31f25f95c6ae9752392daa38f45c59e57593a4def26427cbda25747938d0a3",
          "parentId": "99e19212a3dd1f3951464cbeb42e6bc447b646fe20e362653009be31b8ac756d",
          "adProofsRoot": "7bd5d1c0df436e1e175df31a0272013ca93f2305a1bddea99d6b607ecf24fcac",
          "stateRoot": "000000000000000000000000000000000000000000000000000000000000000000",
          "transactionsRoot": "024562cb92738cbe9f5345b6d78b0272f4823692d2a16a11d80459c6d61f7860",
          "timestamp": 0,
          "nBits": 16842752,
          "height": 8,
          "extensionHash": "696a076089090b128c4856a17e2e23ed2788d074fe09e9975e2ee799f79e9743",
          "powSolutions": {
            "pk": "038b0f29a60fa8d7e1aeafbe512288a6c6bc696547bbf8247db23c95e83014513c",
            "w": "03a0f176159c0895ee4cc64149ff3fa8a0ccf8cf15b884fb0bfaabc0bae2355b10",
            "n": "8000000000000000",
            "d": "21099867708510879837487107380901512552573946537697225889752133246839759121455"
            },
            "votes": "000000"
            },
            "interlinks": [
            "e57483be3653936848f646ebd44e8ea7311d3d8fbc07b4c900a1b7edcc0b9a74",
            "b2525defb4a3ab2f0ffdad6f37ff5a9e330eae2f00ec0c19f1aef234369bd408",
            "99e19212a3dd1f3951464cbeb42e6bc447b646fe20e362653009be31b8ac756d"
            ],
            "interlinksProof": {
            "indices": [
                {
                "index": 0,
                "digest": "e489b3601239d37b066aa53b6640758c9774334dde616d43fb8288e1fdb4f083"
                },
                {
                "index": 1,
                "digest": "76387dd1010dbfc7e318fb7989fe3013099ca0ed4fc6176471815ab116e3542a"
                },
                {
                "index": 2,
                "digest": "c61ce82b2b3bfe369daaa51e5df8070364d68e0cb9a3dfa6ebf79e3bfb89d9fa"
                }
                ],
                "proofs": [
                    {
                    "digest": "",
                    "side": 1
                    }
                ]
             }
          }
          """;
        let header = """
          {
        "version": 1,
        "id": "fa31f25f95c6ae9752392daa38f45c59e57593a4def26427cbda25747938d0a3",
        "parentId": "99e19212a3dd1f3951464cbeb42e6bc447b646fe20e362653009be31b8ac756d",
        "adProofsRoot": "7bd5d1c0df436e1e175df31a0272013ca93f2305a1bddea99d6b607ecf24fcac",
        "stateRoot": "000000000000000000000000000000000000000000000000000000000000000000",
        "transactionsRoot": "024562cb92738cbe9f5345b6d78b0272f4823692d2a16a11d80459c6d61f7860",
        "timestamp": 0,
        "nBits": 16842752,
        "height": 8,
        "extensionHash": "696a076089090b128c4856a17e2e23ed2788d074fe09e9975e2ee799f79e9743",
        "powSolutions": {
            "pk": "038b0f29a60fa8d7e1aeafbe512288a6c6bc696547bbf8247db23c95e83014513c",
            "w": "03a0f176159c0895ee4cc64149ff3fa8a0ccf8cf15b884fb0bfaabc0bae2355b10",
            "n": "8000000000000000",
            "d": "21099867708510879837487107380901512552573946537697225889752133246839759121455"
        },
        "votes": "000000"
        }
        """
        let popow = try PoPowHeader(withJson: json)
        XCTAssert((try popow.getInterlinks()).len() == 3)
        XCTAssert((try popow.getHeader()) == (try BlockHeader(withJson: header)))
        XCTAssert(popow.checkInterlinksProof())
    }
}

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
          "id": "263faffad5a9547bf7269daeebf159df421bfd3016d970862847598cebd3a60e",
          "parentId": "c1087daff44c6a6171737e816edc317d3158ce66fa7c1849441d60611a19102c",
          "adProofsRoot": "55fc89a2437c7508ad693acec80979413b4b1aeb15e30454595783e8c0b955c4",
          "stateRoot": "000000000000000000000000000000000000000000000000000000000000000000",
          "transactionsRoot": "024562cb92738cbe9f5345b6d78b0272f4823692d2a16a11d80459c6d61f7860",
          "timestamp": 0,
          "nBits": 16842752,
          "height": 10,
          "extensionHash": "e1681a85426aa0dd1dfe0f7a00ded5c533c3588ec000a82acc215fd99811d68f",
          "powSolutions": {
          "pk": "038b0f29a60fa8d7e1aeafbe512288a6c6bc696547bbf8247db23c95e83014513c",
          "w": "02204860079465845e810eb776527a14f05e79246922376d1bf1f9f297db19903f",
          "n": "8000000000000000",
          "d": "26629515771172372057548230745860429691028460268159289102352916816108206936640"
          },
          "votes": "000000"
          },
          "interlinks": [
          "d59fdee4f758ef38943a062bb2458778cd4382027a4cb1c15dc9506042e84dd6",
          "66a138771903c442c58839d99b4d20db255b1fae55c95ad8cad65498025a0881",
          "66a138771903c442c58839d99b4d20db255b1fae55c95ad8cad65498025a0881",
          "f89347d89e17e1568d16d4a785c187fd0936cf0fa19110a2cb7eaa63146c59c5",
          "f89347d89e17e1568d16d4a785c187fd0936cf0fa19110a2cb7eaa63146c59c5",
          "f89347d89e17e1568d16d4a785c187fd0936cf0fa19110a2cb7eaa63146c59c5"
          ],
          "interlinks_proof":
          {"indices":[{"index":0,"digest":[-30,75,59,92,-34,-56,57,-107,39,-109,-77,24,-75,-9,94,-47,61,118,-21,-80,58,-49,-64,-51,110,111,109,65,102,48,-97,-101]},
          {"index":1,"digest":[-43,-58,-32,74,110,-126,-91,65,125,-112,81,-99,82,52,-89,-54,-48,-69,33,-47,39,104,-80,89,105,-34,96,59,1,125,48,-67]},
          {"index":2,"digest":[59,90,-94,-64,84,126,-84,-116,-120,95,-24,-57,40,100,108,123,12,-11,-83,26,113,66,21,-70,-75,-63,-92,37,49,-32,4,-24]}],
          "proofs":[{"digest":[],"side":1}]}}
          """;
        let header = """
          {
          "version": 1,
          "id": "263faffad5a9547bf7269daeebf159df421bfd3016d970862847598cebd3a60e",
          "parentId": "c1087daff44c6a6171737e816edc317d3158ce66fa7c1849441d60611a19102c",
          "adProofsRoot": "55fc89a2437c7508ad693acec80979413b4b1aeb15e30454595783e8c0b955c4",
          "stateRoot": "000000000000000000000000000000000000000000000000000000000000000000",
          "transactionsRoot": "024562cb92738cbe9f5345b6d78b0272f4823692d2a16a11d80459c6d61f7860",
          "timestamp": 0,
          "nBits": 16842752,
          "height": 10,
          "extensionHash": "e1681a85426aa0dd1dfe0f7a00ded5c533c3588ec000a82acc215fd99811d68f",
          "powSolutions": {
          "pk": "038b0f29a60fa8d7e1aeafbe512288a6c6bc696547bbf8247db23c95e83014513c",
          "w": "02204860079465845e810eb776527a14f05e79246922376d1bf1f9f297db19903f",
          "n": "8000000000000000",
          "d": "26629515771172372057548230745860429691028460268159289102352916816108206936640"
          },
          "votes": "000000"
           }
        """
        let popow = try PoPowHeader(withJson: json)
        XCTAssert((try popow.getInterlinks()).len() == 6)
        XCTAssert((try popow.getHeader()) == (try BlockHeader(withJson: header)))
    }
}

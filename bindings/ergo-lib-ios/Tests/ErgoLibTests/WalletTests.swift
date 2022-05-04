import XCTest

@testable import ErgoLib
@testable import ErgoLibC

final class WalletTests: XCTestCase {

    func testGenerateMnemonicEnglish() throws {
        let strengthsCounts: [UInt32: Int] = [
            128: 12,
            160: 15,
            192: 18,
            224: 21,
            256: 24,
        ]
        for (strength, expectedWordCount) in strengthsCounts {
            let mg = try MnemonicGenerator(language: "english", strength: strength)
            let mnemonic = try mg.generate()
            let words = mnemonic.components(separatedBy: " ")
            XCTAssertEqual(words.count, expectedWordCount)
            for word in words {
                XCTAssert(word.count > 0)
            }
        }
    }

    func testGenerateMnemonicEnglishFromEntropy() throws {
        let mg = try MnemonicGenerator(language: "english", strength: 128)
        let bytes: [UInt8] = [39, 77, 111, 111, 102, 33, 39, 0, 39, 77, 111, 111, 102, 33, 39, 0]
        let mnemonic = try mg.generateFromEntropy(entropy: bytes)
        XCTAssertEqual(
            mnemonic, "chef hidden swift slush bar length outdoor pupil hunt country endorse accuse"
        )
    }
}

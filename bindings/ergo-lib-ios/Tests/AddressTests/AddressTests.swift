import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class AddressTests: XCTestCase {
    func testTestnetAddress() throws {
        let validTestnetAddrStr = "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN"
        let testnetAddr = try ErgoLib.Address(withTestnetAddress: validTestnetAddrStr)
        XCTAssertEqual(try testnetAddr.toBase58(networkPrefix: NetworkPrefix.Testnet), validTestnetAddrStr)
    }
    
    func testMainnetAddress() throws {
        let validMainnetAddrStr = "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA"
        let mainnetAddr = try ErgoLib.Address(withMainnetAddress: validMainnetAddrStr)
        XCTAssertEqual(try mainnetAddr.toBase58(networkPrefix: NetworkPrefix.Mainnet), validMainnetAddrStr)
    }
    
    func testBase58Address() throws {
        let validMainnetAddrStr = "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA"
        let fromBase58Addr = try ErgoLib.Address(withBase58Address: validMainnetAddrStr)
        XCTAssertEqual(try fromBase58Addr.toBase58(networkPrefix: NetworkPrefix.Mainnet), validMainnetAddrStr)
    }
    
    func testInvalidAddress() throws {
        XCTAssertThrowsError(try ErgoLib.Address(withTestnetAddress: "sss"))
    }
}

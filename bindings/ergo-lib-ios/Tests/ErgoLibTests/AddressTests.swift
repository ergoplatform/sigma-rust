import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class AddressTests: XCTestCase {
    func testTestnetAddress() throws {
        let p2pkAddrStr = "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN"
        let p2pkAddr = try ErgoLib.Address(withTestnetAddress: p2pkAddrStr)
        XCTAssertEqual(p2pkAddr.toBase58(networkPrefix: NetworkPrefix.Testnet), p2pkAddrStr)
        XCTAssertEqual(p2pkAddr.typePrefix(), AddressTypePrefix.P2Pk)
        
        let p2shAddrStr = "rbcrmKEYduUvADj9Ts3dSVSG27h54pgrq5fPuwB"
        let p2shAddr = try ErgoLib.Address(withTestnetAddress: p2shAddrStr)
        XCTAssertEqual(p2shAddr.toBase58(networkPrefix: NetworkPrefix.Testnet), p2shAddrStr)
        XCTAssertEqual(p2shAddr.typePrefix(), AddressTypePrefix.Pay2Sh)
        
    }
    
    func testMainnetAddress() throws {
        let p2pkAddrStr = "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA"
        let p2pkAddr = try ErgoLib.Address(withMainnetAddress: p2pkAddrStr)
        XCTAssertEqual(p2pkAddr.toBase58(networkPrefix: NetworkPrefix.Mainnet), p2pkAddrStr)
        XCTAssertEqual(p2pkAddr.typePrefix(), AddressTypePrefix.P2Pk)
        
        let p2shAddrStr = "8UApt8czfFVuTgQmMwtsRBZ4nfWquNiSwCWUjMg"
        let p2shAddr = try ErgoLib.Address(withMainnetAddress: p2shAddrStr)
        XCTAssertEqual(p2shAddr.toBase58(networkPrefix: NetworkPrefix.Mainnet), p2shAddrStr)
        XCTAssertEqual(p2shAddr.typePrefix(), AddressTypePrefix.Pay2Sh)
    }
    
    func testBase58Address() throws {
        let validMainnetAddrStr = "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA"
        let fromBase58Addr = try ErgoLib.Address(withBase58Address: validMainnetAddrStr)
        XCTAssertEqual(fromBase58Addr.toBase58(networkPrefix: NetworkPrefix.Mainnet), validMainnetAddrStr)
    }
    
    func testInvalidAddress() throws {
        XCTAssertThrowsError(try ErgoLib.Address(withTestnetAddress: "sss"))
    }
}

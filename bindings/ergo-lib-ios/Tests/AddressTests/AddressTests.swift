import XCTest
@testable import ErgoLib

final class AddressTests: XCTestCase {
    func testAddress() throws {
        let validTestnetAddr = "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN"
        XCTAssertNoThrow(try ErgoLib.Address(withTestnetAddress: validTestnetAddr))
        
        let validMainnetAddr = "9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA"
        XCTAssertNoThrow(try ErgoLib.Address(withMainnetAddress: validMainnetAddr))
        
        XCTAssertThrowsError(try ErgoLib.Address(withTestnetAddress: "sss"))
    }
}

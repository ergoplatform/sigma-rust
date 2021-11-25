import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class BoxBuilderTests: XCTestCase {
    static func genElements() throws -> (Contract, BoxValue) {
        let addr = try Address(withTestnetAddress: "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        let contract = try Contract(payToAddress: addr)
        let boxValue = try BoxValue(fromInt64: Int64(10000000))
        return (contract, boxValue)
    }
    
    func testSimpleBox() throws {
        let (contract, boxValue) = try BoxBuilderTests.genElements()
        XCTAssertNoThrow(try ErgoBoxCandidateBuilder(boxValue: boxValue, contract: contract, creationHeight: UInt32(0)))
    }
    
    func testSetRegister() throws {
        let (contract, boxValue) = try BoxBuilderTests.genElements()
        let constant = try Constant(withInt32: Int32(1))
        let builder = try ErgoBoxCandidateBuilder(boxValue: boxValue, contract: contract, creationHeight: UInt32(0))
            .setRegisterValue(registerId: NonMandatoryRegisterId.R4, constant: constant)
        XCTAssertEqual(try builder.getRegisterValue(registerId: NonMandatoryRegisterId.R4)!, constant)
        let box = try builder.build()
        XCTAssertEqual(try box.getRegisterValue(registerId: NonMandatoryRegisterId.R4)!, constant)
    }
    
    func testDeleteRegister() throws {
        let (contract, boxValue) = try BoxBuilderTests.genElements()
        let constant = try Constant(withInt32: Int32(1))
        var builder = try ErgoBoxCandidateBuilder(boxValue: boxValue, contract: contract, creationHeight: UInt32(0))
            .setRegisterValue(registerId: NonMandatoryRegisterId.R4, constant: constant)
        XCTAssertEqual(try builder.getRegisterValue(registerId: NonMandatoryRegisterId.R4)!, constant)
        
        // Now delete register value
        builder = try builder.deleteRegisterValue(registerId: NonMandatoryRegisterId.R4)
        XCTAssertNil(try builder.getRegisterValue(registerId: NonMandatoryRegisterId.R4))
        let box = try builder.build()
        XCTAssertNil(try box.getRegisterValue(registerId: NonMandatoryRegisterId.R4))
    }
}


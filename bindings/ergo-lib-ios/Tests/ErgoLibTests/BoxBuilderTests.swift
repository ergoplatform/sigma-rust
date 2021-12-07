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
        XCTAssertNoThrow(ErgoBoxCandidateBuilder(boxValue: boxValue, contract: contract, creationHeight: UInt32(0)))
    }
    
    func testSetRegister() throws {
        let (contract, boxValue) = try BoxBuilderTests.genElements()
        let constant = Constant(withInt32: Int32(1))
        let builder = ErgoBoxCandidateBuilder(boxValue: boxValue, contract: contract, creationHeight: UInt32(0))
            .setRegisterValue(registerId: NonMandatoryRegisterId.R4, constant: constant)
        XCTAssertEqual(builder.getRegisterValue(registerId: NonMandatoryRegisterId.R4)!, constant)
        let box = try builder.build()
        XCTAssertEqual(box.getRegisterValue(registerId: NonMandatoryRegisterId.R4)!, constant)
    }
    
    func testDeleteRegister() throws {
        let (contract, boxValue) = try BoxBuilderTests.genElements()
        let constant = Constant(withInt32: Int32(1))
        var builder = ErgoBoxCandidateBuilder(boxValue: boxValue, contract: contract, creationHeight: UInt32(0))
            .setRegisterValue(registerId: NonMandatoryRegisterId.R4, constant: constant)
        XCTAssertEqual(builder.getRegisterValue(registerId: NonMandatoryRegisterId.R4)!, constant)
        
        // Now delete register value
        builder = builder.deleteRegisterValue(registerId: NonMandatoryRegisterId.R4)
        XCTAssertNil(builder.getRegisterValue(registerId: NonMandatoryRegisterId.R4))
        let box = try builder.build()
        XCTAssertNil(box.getRegisterValue(registerId: NonMandatoryRegisterId.R4))
    }
}


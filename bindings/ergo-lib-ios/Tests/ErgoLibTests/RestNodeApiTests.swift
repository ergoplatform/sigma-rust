
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiTests: XCTestCase {

    func testGetInfo() throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let nodeInfo = try RestNodeApi.getInfo(nodeConf: nodeConf)
        XCTAssertEqual(nodeInfo.getName(), "ergo-mainnet-4.0.16.1")
    }
}

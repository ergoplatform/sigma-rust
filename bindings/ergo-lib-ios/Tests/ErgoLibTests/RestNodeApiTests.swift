
import XCTest
@testable import ErgoLib
@testable import ErgoLibC

final class RestNodeApiTests: XCTestCase {

    func testGetInfo() throws {
        let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053")
        let restNodeApi = try RestNodeApi()
        let nodeInfo = try restNodeApi.getInfo(nodeConf: nodeConf)
        XCTAssert(!nodeInfo.getName().isEmpty)
    }

    // need macOS 12.0 on Github GA
    /* func testGetInfoAsync() throws { */
    /*     let nodeConf = try NodeConf(withAddrString: "213.239.193.208:9053") */
    /*     let restNodeApi = try RestNodeApi() */
    /*     XCTAssertNoThrow(Task(priority: .medium) { */
    /*         let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf) */
    /*         XCTAssert(!nodeInfo.getName().isEmpty) */
    /*     }) */
    /*     // test of re-using of tokio runtime */
    /*     XCTAssertNoThrow(Task(priority: .medium) { */
    /*         let nodeInfo = try await restNodeApi.getInfoAsync(nodeConf: nodeConf) */
    /*         XCTAssert(!nodeInfo.getName().isEmpty) */
    /*     }) */
    /* } */
}

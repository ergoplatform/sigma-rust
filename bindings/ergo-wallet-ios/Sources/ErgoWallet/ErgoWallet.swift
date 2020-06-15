import Foundation
import ErgoWalletC

struct UnspentInputBoxes {
    private init() {}
    static func from_json(json: String) throws -> UnspentInputBoxesPtr {
        var unspentInputBoxesPtr: UnspentInputBoxesPtr?
        let _ = json.withCString { cs in
            ergo_wallet_unspent_input_boxes_from_json(cs, &unspentInputBoxesPtr)
        }
        return unspentInputBoxesPtr!
    }
}


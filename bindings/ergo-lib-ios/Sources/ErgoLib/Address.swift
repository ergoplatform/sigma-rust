import Foundation
import ErgoLibC

class Address {
    internal var pointer: AddressPtr

    init(withTestnetAddress addressStr: String) throws {
        self.pointer = try Address.fromTestnetAddress(addressStr: addressStr)
    }

    init(withMainnetAddress addressStr: String) throws {
        self.pointer = try Address.fromMainnetAddress(addressStr: addressStr)
    }
    
    init(withBase58Address addressStr: String) throws {
        self.pointer = try Address.fromBase58(addressStr: addressStr)
    }
    
    func toBase58(networkPrefix: NetworkPrefix) throws -> String {
        var cStr: UnsafePointer<CChar>?
        let error = ergo_wallet_address_to_base58(self.pointer, networkPrefix.rawValue, &cStr)
        try checkError(error)
        let str = String(cString: cStr!)
        ergo_wallet_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    private static func fromTestnetAddress(addressStr: String) throws -> AddressPtr {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_wallet_address_from_testnet(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    private static func fromMainnetAddress(addressStr: String) throws -> AddressPtr {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_wallet_address_from_mainnet(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    private static func fromBase58(addressStr: String) throws -> AddressPtr {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_wallet_address_from_base58(cs, &ptr)
        }
        try checkError(error)
        return ptr!
    }
    
    deinit {
        ergo_wallet_address_delete(self.pointer)
    }
}
enum NetworkPrefix: UInt8 {
    case Mainnet = 0
    case Testnet = 16
}

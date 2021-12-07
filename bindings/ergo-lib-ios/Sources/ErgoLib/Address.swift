import Foundation
import ErgoLibC

/**
 * An address is a short string corresponding to some script used to protect a box. Unlike (string-encoded) binary
 * representation of a script, an address has some useful characteristics:
 *
 * - Integrity of an address could be checked., as it is incorporating a checksum.
 * - A prefix of address is showing network and an address type.
 * - An address is using an encoding (namely, Base58) which is avoiding similarly l0Oking characters, friendly to
 * double-clicking and line-breaking in emails.
 *
 *
 *
 * An address is encoding network type, address type, checksum, and enough information to watch for a particular scripts.
 *
 * Possible network types are:
 * Mainnet - 0x00
 * Testnet - 0x10
 *
 * For an address type, we form content bytes as follows:
 *
 * P2PK - serialized (compressed) public key
 * P2SH - first 192 bits of the Blake2b256 hash of serialized script bytes
 * P2S  - serialized script
 *
 * Address examples for testnet:
 *
 * 3   - P2PK (3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN)
 * ?   - P2SH (rbcrmKEYduUvADj9Ts3dSVSG27h54pgrq5fPuwB)
 * ?   - P2S (Ms7smJwLGbUAjuWQ)
 *
 * for mainnet:
 *
 * 9  - P2PK (9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA)
 * ?  - P2SH (8UApt8czfFVuTgQmMwtsRBZ4nfWquNiSwCWUjMg)
 * ?  - P2S (4MQyML64GnzMxZgm, BxKBaHkvrTvLZrDcZjcsxsF7aSsrN73ijeFZXtbj4CXZHHcvBtqSxQ)
 *
 *
 * Prefix byte = network type + address type
 *
 * checksum = blake2b256(prefix byte ++ content bytes)
 *
 * address = prefix byte ++ content bytes ++ checksum
 *
 */
class Address {
    internal var pointer: AddressPtr

    /// Decode (base58) testnet address from string, checking that address is from the testnet
    init(withTestnetAddress addressStr: String) throws {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_lib_address_from_testnet(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }

    /// Decode (base58) mainnet address from string, checking that address is from the mainnet
    init(withMainnetAddress addressStr: String) throws {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_lib_address_from_mainnet(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Decode (base58) address from string without checking the network prefix
    init(withBase58Address addressStr: String) throws {
        var ptr: AddressPtr?
        let error = addressStr.withCString { cs in
            ergo_lib_address_from_base58(cs, &ptr)
        }
        try checkError(error)
        self.pointer = ptr!
    }
    
    /// Takes ownership of an existing ``AddressPtr``. Note: we must ensure that no other instance
    /// of ``Address`` can hold this pointer.
    internal init(withRawPointer ptr: AddressPtr) {
        self.pointer = ptr
    }
    
    /// Encode (base58) address
    func toBase58(networkPrefix: NetworkPrefix) -> String {
        var cStr: UnsafePointer<CChar>?
        ergo_lib_address_to_base58(self.pointer, networkPrefix.rawValue, &cStr)
        let str = String(cString: cStr!)
        ergo_lib_delete_string(UnsafeMutablePointer(mutating: cStr))
        return str
    }
    
    /// Get the type of the address
    func typePrefix() -> AddressTypePrefix {
        let value = ergo_lib_address_type_prefix(self.pointer)
        return AddressTypePrefix(rawValue: value)!
    }
    
    deinit {
        ergo_lib_address_delete(self.pointer)
    }
}

/// Network type
enum NetworkPrefix: UInt8 {
    case Mainnet = 0
    case Testnet = 16
}

/// Address types
enum AddressTypePrefix: UInt8 {
    /// 0x01 - Pay-to-PublicKey(P2PK) address
    case P2Pk = 1
    /// 0x02 - Pay-to-Script-Hash(P2SH)
    case Pay2Sh = 2
    /// 0x03 - Pay-to-Script(P2S)
    case Pay2S = 3
}

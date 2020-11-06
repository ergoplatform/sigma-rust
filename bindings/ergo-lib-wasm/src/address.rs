//! Address types

use ergo_lib::chain;
use ergo_lib::{
    serialization::SigmaSerializable,
    sigma_protocol::{dlog_group::EcPoint, sigma_boolean::ProveDlog},
};
use wasm_bindgen::prelude::*;

use crate::ergo_tree::ErgoTree;

/// Network type
#[wasm_bindgen]
#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NetworkPrefix {
    /// Mainnet
    Mainnet = 0,
    /// Testnet
    Testnet = 16,
}

impl From<NetworkPrefix> for chain::address::NetworkPrefix {
    fn from(v: NetworkPrefix) -> Self {
        use chain::address::NetworkPrefix::*;
        match v {
            NetworkPrefix::Mainnet => Mainnet,
            NetworkPrefix::Testnet => Testnet,
        }
    }
}

impl From<chain::address::NetworkPrefix> for NetworkPrefix {
    fn from(v: chain::address::NetworkPrefix) -> Self {
        use NetworkPrefix::*;
        match v {
            chain::address::NetworkPrefix::Mainnet => Mainnet,
            chain::address::NetworkPrefix::Testnet => Testnet,
        }
    }
}

/// Address types
#[wasm_bindgen]
#[repr(u8)]
pub enum AddressTypePrefix {
    /// 0x01 - Pay-to-PublicKey(P2PK) address
    P2PK = 1,
    /// 0x02 - Pay-to-Script-Hash(P2SH)
    Pay2SH = 2,
    /// 0x03 - Pay-to-Script(P2S)
    Pay2S = 3,
}

impl From<AddressTypePrefix> for chain::address::AddressTypePrefix {
    fn from(v: AddressTypePrefix) -> Self {
        use chain::address::AddressTypePrefix::*;
        match v {
            AddressTypePrefix::P2PK => P2PK,
            AddressTypePrefix::Pay2SH => Pay2SH,
            AddressTypePrefix::Pay2S => Pay2S,
        }
    }
}

impl From<chain::address::AddressTypePrefix> for AddressTypePrefix {
    fn from(v: chain::address::AddressTypePrefix) -> Self {
        use AddressTypePrefix::*;
        match v {
            chain::address::AddressTypePrefix::P2PK => P2PK,
            chain::address::AddressTypePrefix::Pay2SH => Pay2SH,
            chain::address::AddressTypePrefix::Pay2S => Pay2S,
        }
    }
}

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
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Address(chain::address::Address);

#[wasm_bindgen]
impl Address {
    /// Create a P2PK address from an ergo tree if ProveDlog is the root of the tree, otherwise returns an error
    pub fn new_p2pk(ergo_tree: &ErgoTree) -> Result<Address, JsValue> {
        chain::address::Address::new_p2pk(&ergo_tree.clone().into())
            .map(Address)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Decode (base58) testnet address from string
    pub fn from_testnet_str(s: &str) -> Result<Address, JsValue> {
        chain::address::AddressEncoder::new(chain::address::NetworkPrefix::Testnet)
            .parse_address_from_str(s)
            .map(Address)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Decode (base58) mainnet address from string
    pub fn from_mainnet_str(s: &str) -> Result<Address, JsValue> {
        chain::address::AddressEncoder::new(chain::address::NetworkPrefix::Mainnet)
            .parse_address_from_str(s)
            .map(Address)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Decode (base58) address from string without checking the network prefix
    pub fn from_base58(s: &str) -> Result<Address, JsValue> {
        chain::address::AddressEncoder::unchecked_parse_address_from_str(s)
            .map(Address)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Encode (base58) address
    pub fn to_base58(&self, network_prefix: NetworkPrefix) -> String {
        chain::address::AddressEncoder::encode_address(network_prefix.into(), &self.0)
    }

    /// Get the type of the address
    pub fn address_type_prefix(&self) -> AddressTypePrefix {
        self.0.address_type_prefix().into()
    }

    /// Create an address from a public key
    pub fn from_public_key(bytes: &[u8]) -> Result<Address, JsValue> {
        EcPoint::sigma_parse_bytes(bytes.to_vec())
            .map(|point| chain::address::Address::P2PK(ProveDlog::new(point)))
            .map(Address)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }
}

impl Into<chain::address::Address> for Address {
    fn into(self) -> chain::address::Address {
        self.0
    }
}

impl From<chain::address::Address> for Address {
    fn from(a: chain::address::Address) -> Self {
        Address(a)
    }
}

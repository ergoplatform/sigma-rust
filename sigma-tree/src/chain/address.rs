use crate::{
    ast::{Constant, Expr},
    ecpoint::EcPoint,
    sigma_protocol::{ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree, SigmaProp},
    ErgoTree,
};
use std::fmt;
use std::rc::Rc;

/// Address types
pub enum AddressTypePrefix {
    /// 0x01 - Pay-to-PublicKey(P2PK) address
    P2PKAddress = 1,
    /// 0x02 - Pay-to-Script-Hash(P2SH)
    Pay2SHAddress = 2,
    /// 0x03 - Pay-to-Script(P2S)
    Pay2SAddress = 3,
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
pub trait Address {
    /// address type (P2PK, P2SH or P2S)
    fn address_type_prefix(&self) -> AddressTypePrefix;

    /// script encoded in the address
    fn script(&self) -> ErgoTree;
}

/// P2PK - serialized (compressed) public key
pub struct P2PKAddress {
    /// pubkey (encoded in the address)
    pubkey: ProveDlog,
}

impl P2PKAddress {
    /// create from ProveDlog
    pub fn new(pubkey: ProveDlog) -> P2PKAddress {
        P2PKAddress { pubkey }
    }
}

impl Address for P2PKAddress {
    fn address_type_prefix(&self) -> AddressTypePrefix {
        AddressTypePrefix::P2PKAddress
    }
    fn script(&self) -> ErgoTree {
        ErgoTree::from_proposition(Rc::new(Expr::Const(Constant::sigma_prop(SigmaProp::new(
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(
                self.pubkey.clone(),
            )),
        )))))
    }
}

/// Network type
pub enum NetworkPrefix {
    /// Mainnet
    Mainnet = 0,
    /// Testnet
    Testnet = 16,
}

/// Errors on encoding/decoding of addresses
#[derive(Debug)]
pub enum AddressEncoderError {
    /// Failed to decode Base58
    Base58DecodingError(String),
}

impl fmt::Display for AddressEncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AddressEncoderError::Base58DecodingError(e) => write!(f, "Error({:?}, {})", self, e),
        }
    }
}

/// Encodes/Decodes address to/from string
pub struct AddressEncoder(NetworkPrefix);

impl AddressEncoder {
    /// create a new AddressEncoder for a given network type
    pub fn new(network_prefix: NetworkPrefix) -> AddressEncoder {
        AddressEncoder(network_prefix)
    }

    /// parse address from Base58 encoded string
    pub fn parse_address_from_str(&self, _: &str) -> Result<Box<dyn Address>, AddressEncoderError> {
        // TODO: implement
        Ok(Box::new(P2PKAddress::new(
            ProveDlog::new(EcPoint::random()),
        )))
    }
}

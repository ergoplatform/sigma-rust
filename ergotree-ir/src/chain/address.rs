//! Address types

use crate::ergo_tree::ErgoTreeError;
use crate::ergo_tree::{ErgoTree, ErgoTreeHeader};
use crate::mir::bin_op::BinOpKind::Relation;
use crate::mir::bin_op::{BinOp, RelationOp};
use crate::mir::bool_to_sigma::BoolToSigmaProp;
use crate::mir::calc_blake2b256::CalcBlake2b256;
use crate::mir::coll_slice::Slice;
use crate::mir::constant::Literal::Int;
use crate::mir::constant::{Constant, Literal};
use crate::mir::deserialize_context::DeserializeContext;
use crate::mir::expr::Expr;
use crate::mir::get_var::GetVar;
use crate::mir::sigma_and::SigmaAnd;
use crate::mir::value::CollKind;
use crate::mir::value::NativeColl::CollByte;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializationError;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use crate::sigma_protocol::sigma_boolean::SigmaProp;
use crate::types::stype::SType;
use ergo_chain_types::EcPoint;

use sigma_util::hash::blake2b256_hash;
use sigma_util::AsVecU8;
use std::convert::{TryFrom, TryInto};
use thiserror::Error;

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
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Address {
    /// serialized (compressed) public key
    P2Pk(ProveDlog),
    /// serialized script
    P2S(Vec<u8>),
    /// hash of serialized script (192 bit)
    P2SH([u8; 24]),
}

impl Address {
    /// Create a P2PK address from serialized PK bytes(EcPoint/GroupElement)
    pub fn p2pk_from_pk_bytes(bytes: &[u8]) -> Result<Address, SigmaParsingError> {
        EcPoint::sigma_parse_bytes(bytes)
            .map(ProveDlog::from)
            .map(Address::P2Pk)
    }

    /// Re-create the address from ErgoTree that was built from the address
    ///
    /// At some point in the past a user entered an address from which the ErgoTree was built.
    /// Re-create the address from this ErgoTree.
    /// `tree` - ErgoTree that was created from an Address
    pub fn recreate_from_ergo_tree(tree: &ErgoTree) -> Result<Address, AddressError> {
        match tree.proposition() {
            Ok(expr) => Ok(match expr {
                Expr::Const(Constant {
                    tpe: SType::SSigmaProp,
                    v,
                }) => match ProveDlog::try_from(v).map(Address::P2Pk) {
                    Ok(p2pk) => p2pk,
                    Err(_) => Address::P2S(tree.sigma_serialize_bytes()?),
                },
                Expr::SigmaAnd(SigmaAnd { items }) => {
                    if let [Expr::BoolToSigmaProp(BoolToSigmaProp { input }), Expr::DeserializeContext(DeserializeContext { tpe, id })] =
                        items.as_slice()
                    {
                        if let (Expr::BinOp(BinOp { kind, left, right }), SType::SSigmaProp, 1) =
                            (*input.clone(), tpe.clone(), id)
                        {
                            if let (
                                Relation(RelationOp::Eq),
                                Expr::Slice(Slice { input, from, until }),
                                Expr::Const(Constant { v, .. }),
                            ) = (kind, *left, *right)
                            {
                                if let (
                                    Expr::CalcBlake2b256(..),
                                    Expr::Const(Constant {
                                        tpe: from_tpe,
                                        v: from_v,
                                    }),
                                    Expr::Const(Constant {
                                        tpe: until_tpe,
                                        v: until_v,
                                    }),
                                    Literal::Coll(CollKind::NativeColl(bytes)),
                                ) = (*input, *from, *until, v)
                                {
                                    if let (
                                        SType::SInt,
                                        Int(0),
                                        SType::SInt,
                                        Int(24),
                                        CollByte(v),
                                    ) = (from_tpe, from_v, until_tpe, until_v, bytes)
                                    {
                                        let script_hash = v.as_vec_u8();
                                        return <[u8; 24]>::try_from(script_hash)
                                                .map(Address::P2SH)
                                                .map_err(|_| {
                                                    AddressError::UnexpectedErgoTree(
                                                        tree.clone(),
                                                        String::from("Failed to create P2SH address, invalid script hash."),
                                                    )
                                                });
                                    }
                                }
                            }
                        }
                    }
                    Address::P2S(tree.sigma_serialize_bytes()?)
                }
                _ => Address::P2S(tree.sigma_serialize_bytes()?),
            }),
            Err(_) => Ok(Address::P2S(tree.sigma_serialize_bytes()?)),
        }
    }

    /// address type prefix (for encoding)
    pub fn address_type_prefix(&self) -> AddressTypePrefix {
        match self {
            Address::P2Pk(_) => AddressTypePrefix::P2Pk,
            Address::P2S(_) => AddressTypePrefix::Pay2S,
            Address::P2SH(_) => AddressTypePrefix::Pay2Sh,
        }
    }

    /// Returns underlying value for each address type: (serialized EcPoint for P2PK, stored bytes for P2SH and P2S)
    pub fn content_bytes(&self) -> Vec<u8> {
        match self {
            Address::P2Pk(prove_dlog) => {
                #[allow(clippy::unwrap_used)]
                // Since ProveDlog is a simple IR node we can be sure no other errors besides OOM could be here
                prove_dlog.h.sigma_serialize_bytes().unwrap()
            }
            Address::P2S(bytes) => bytes.clone(),
            Address::P2SH(bytes) => (*bytes).into(),
        }
    }

    /// script encoded in the address
    pub fn script(&self) -> Result<ErgoTree, SigmaParsingError> {
        match self {
            Address::P2Pk(prove_dlog) => {
                #[allow(clippy::unwrap_used)]
                // Since ProveDlog is a simple IR node we can be sure no other errors besides OOM could be here
                Ok(ErgoTree::try_from(Expr::Const(
                    SigmaProp::new(SigmaBoolean::ProofOfKnowledge(
                        SigmaProofOfKnowledgeTree::ProveDlog(prove_dlog.clone()),
                    ))
                    .into(),
                ))
                .unwrap())
            }
            Address::P2S(bytes) => ErgoTree::sigma_parse_bytes(bytes),
            Address::P2SH(script_hash) => {
                let get_var_expr = Expr::GetVar(GetVar {
                    var_id: 1,
                    var_tpe: SType::SColl(Box::new(SType::SByte)),
                });
                let hash_expr = Expr::CalcBlake2b256(CalcBlake2b256 {
                    input: Box::new(get_var_expr),
                });
                let slice_expr = Expr::Slice(Slice {
                    input: Box::new(hash_expr),
                    from: Box::new(0i32.into()),
                    until: Box::new(24i32.into()),
                });
                let hash_equals = Expr::BinOp(BinOp {
                    kind: Relation(RelationOp::Eq),
                    left: Box::new(slice_expr),
                    right: Box::new(Expr::Const(Constant::from(script_hash.to_vec()))),
                });
                let script_is_correct = Expr::DeserializeContext(DeserializeContext {
                    tpe: SType::SSigmaProp,
                    id: 1,
                });
                let sigma_prop = Expr::BoolToSigmaProp(BoolToSigmaProp {
                    input: Box::from(hash_equals),
                });
                let and_expr = Expr::SigmaAnd(SigmaAnd::new(vec![sigma_prop, script_is_correct])?);

                match ErgoTree::new(ErgoTreeHeader::v0(false), &and_expr) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(SigmaParsingError::Misc(String::from(
                        "P2SH ErgoTree creation failed.",
                    ))),
                }
            }
        }
    }
}

/// Combination of an Address with a network
/// These two combined together form a base58 encoding
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(try_from = "String"), serde(into = "String"))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NetworkAddress {
    network: NetworkPrefix,
    address: Address,
}

impl NetworkAddress {
    /// create a new NetworkAddress for a given network type
    pub fn new(network: NetworkPrefix, address: &Address) -> NetworkAddress {
        NetworkAddress {
            address: address.clone(),
            network,
        }
    }

    /// Encode (base58) address
    pub fn to_base58(&self) -> String {
        AddressEncoder::encode_address_as_string(self.network(), &self.address)
    }

    /// Get the type of the address
    pub fn network(&self) -> NetworkPrefix {
        self.network
    }

    /// Get the type of the address
    pub fn address(&self) -> Address {
        self.address.clone()
    }
}

impl TryFrom<String> for NetworkAddress {
    type Error = AddressEncoderError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        AddressEncoder::unchecked_parse_network_address_from_str(&s)
    }
}

impl From<NetworkAddress> for String {
    fn from(na: NetworkAddress) -> Self {
        na.to_base58()
    }
}

/// Errors for Address processing
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum AddressError {
    /// Unexpected ErgoTree encountered
    #[error("Unexpected ErgoTree: {0:?}, \n reason: {1}")]
    UnexpectedErgoTree(ErgoTree, String),
    /// ErgoTree parsing error
    #[error("ErgoTree error: {0}")]
    ErgoTreeError(#[from] ErgoTreeError),
}

impl From<SigmaSerializationError> for AddressError {
    fn from(e: SigmaSerializationError) -> Self {
        ErgoTreeError::RootSerializationError(e).into()
    }
}

/// Address types
pub enum AddressTypePrefix {
    /// 0x01 - Pay-to-PublicKey(P2PK) address
    P2Pk = 1,
    /// 0x02 - Pay-to-Script-Hash(P2SH)
    Pay2Sh = 2,
    /// 0x03 - Pay-to-Script(P2S)
    Pay2S = 3,
}

impl TryFrom<u8> for AddressTypePrefix {
    type Error = AddressEncoderError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == AddressTypePrefix::P2Pk as u8 => Ok(AddressTypePrefix::P2Pk),
            v if v == AddressTypePrefix::Pay2Sh as u8 => Ok(AddressTypePrefix::Pay2Sh),
            v if v == AddressTypePrefix::Pay2S as u8 => Ok(AddressTypePrefix::Pay2S),
            v => Err(AddressEncoderError::InvalidAddressType(v)),
        }
    }
}

/// Network type
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NetworkPrefix {
    /// Mainnet
    Mainnet = 0,
    /// Testnet
    Testnet = 16,
}

impl TryFrom<u8> for NetworkPrefix {
    type Error = AddressEncoderError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == NetworkPrefix::Mainnet as u8 => Ok(NetworkPrefix::Mainnet),
            v if v == NetworkPrefix::Testnet as u8 => Ok(NetworkPrefix::Testnet),
            _v => Err(AddressEncoderError::InvalidNetwork(
                "Invalid network".to_string(),
            )),
        }
    }
}

/// Errors on encoding/decoding of addresses
#[derive(Error, Eq, PartialEq, Debug, Clone)]
pub enum AddressEncoderError {
    /// Base58 decoding error
    #[error("Base58 decoding error: {0}")]
    Base58DecodingError(String),

    /// Invalid byte array size
    #[error("Invalid size of the decoded byte array")]
    InvalidSize,

    /// address network type does not match network_prefix of this encoder
    #[error("address network type does not match network_prefix of this encoder: {0}")]
    InvalidNetwork(String),

    /// invalid checksum
    #[error("invalid checksum")]
    InvalidChecksum,

    /// invalid address type
    #[error("invalid address type {0}")]
    InvalidAddressType(u8),

    /// deserialization failed
    #[error("deserialization failed {0}")]
    DeserializationFailed(SigmaParsingError),
}

impl From<bs58::decode::Error> for AddressEncoderError {
    fn from(err: bs58::decode::Error) -> Self {
        AddressEncoderError::Base58DecodingError(err.to_string())
    }
}

impl From<SigmaParsingError> for AddressEncoderError {
    fn from(err: SigmaParsingError) -> Self {
        AddressEncoderError::DeserializationFailed(err)
    }
}

/// Encodes/Decodes address to/from string
#[derive(PartialEq, Eq, Debug)]
pub struct AddressEncoder {
    /// Network prefix (network type) of the encoder
    pub network_prefix: NetworkPrefix,
}

impl AddressEncoder {
    const CHECKSUM_LENGTH: usize = 4;
    const MIN_ADDRESS_LENGTH: usize = AddressEncoder::CHECKSUM_LENGTH + 2;

    /// create a new AddressEncoder for a given network type
    pub fn new(network_prefix: NetworkPrefix) -> AddressEncoder {
        AddressEncoder { network_prefix }
    }

    fn is_mainnet_address(head_byte: u8) -> bool {
        head_byte < NetworkPrefix::Testnet as u8
    }

    fn is_testnet_address(head_byte: u8) -> bool {
        head_byte > NetworkPrefix::Testnet as u8
    }

    fn check_head_byte(&self, adr_prefix: u8) -> Result<u8, AddressEncoderError> {
        match self.network_prefix {
            NetworkPrefix::Mainnet if AddressEncoder::is_testnet_address(adr_prefix) => {
                Err(AddressEncoderError::InvalidNetwork(
                    "Expected mainnet address, found testnet".to_string(),
                ))
            }
            NetworkPrefix::Testnet if AddressEncoder::is_mainnet_address(adr_prefix) => {
                Err(AddressEncoderError::InvalidNetwork(
                    "Expected testnet address, found mainnet".to_string(),
                ))
            }
            _ => Ok(adr_prefix),
        }
    }

    fn calc_checksum(bytes: &[u8]) -> [u8; AddressEncoder::CHECKSUM_LENGTH] {
        let v: Vec<u8> = blake2b256_hash(bytes)
            .iter()
            .copied()
            .take(AddressEncoder::CHECKSUM_LENGTH)
            .collect();
        #[allow(clippy::unwrap_used)]
        v.as_slice().try_into().unwrap()
    }

    /// parse address from Base58 encoded string
    pub fn parse_address_from_str(&self, str: &str) -> Result<Address, AddressEncoderError> {
        let bytes = bs58::decode(str).into_vec()?;
        if bytes.len() < AddressEncoder::MIN_ADDRESS_LENGTH {
            return Err(AddressEncoderError::InvalidSize);
        };
        self.check_head_byte(bytes[0])?;
        AddressEncoder::unchecked_parse_address_from_bytes(&bytes)
    }

    /// parse network+address from Base58 encoded string
    pub fn unchecked_parse_network_address_from_str(
        str: &str,
    ) -> Result<NetworkAddress, AddressEncoderError> {
        let bytes = bs58::decode(str).into_vec()?;
        AddressEncoder::unchecked_parse_network_address_from_bytes(&bytes)
    }

    /// parse network+address from Base58 encoded string
    pub fn unchecked_parse_network_address_from_bytes(
        bytes: &[u8],
    ) -> Result<NetworkAddress, AddressEncoderError> {
        if bytes.len() < AddressEncoder::MIN_ADDRESS_LENGTH {
            return Err(AddressEncoderError::InvalidSize);
        };

        let network_prefix = (bytes[0] & 0xF0).try_into()?;
        AddressEncoder::unchecked_parse_address_from_bytes(bytes).map(|addr| NetworkAddress {
            address: addr,
            network: network_prefix,
        })
    }

    /// parse address from Base58 encoded string
    pub fn unchecked_parse_address_from_str(str: &str) -> Result<Address, AddressEncoderError> {
        let bytes = bs58::decode(str).into_vec()?;
        AddressEncoder::unchecked_parse_address_from_bytes(&bytes)
    }

    /// parse address from Base58 encoded string
    pub fn unchecked_parse_address_from_bytes(
        bytes: &[u8],
    ) -> Result<Address, AddressEncoderError> {
        if bytes.len() < AddressEncoder::MIN_ADDRESS_LENGTH {
            return Err(AddressEncoderError::InvalidSize);
        };
        let (without_checksum, checksum) =
            bytes.split_at(bytes.len() - AddressEncoder::CHECKSUM_LENGTH);
        let calculated_checksum = AddressEncoder::calc_checksum(without_checksum);
        if checksum != calculated_checksum {
            return Err(AddressEncoderError::InvalidChecksum);
        };

        let content_bytes: Vec<u8> = without_checksum[1..].to_vec(); // without head_byte
        let address_type = AddressTypePrefix::try_from(bytes[0] & 0xF_u8)?;
        match address_type {
            AddressTypePrefix::P2Pk => Ok(Address::P2Pk(ProveDlog::new(
                EcPoint::sigma_parse_bytes(&content_bytes)?,
            ))),
            AddressTypePrefix::Pay2S => Ok(Address::P2S(content_bytes)),
            AddressTypePrefix::Pay2Sh => match <[u8; 24]>::try_from(content_bytes) {
                Ok(p2sh) => Ok(Address::P2SH(p2sh)),
                _ => Err(AddressEncoderError::InvalidSize),
            },
        }
    }

    /// encode address as Base58 encoded string
    pub fn address_to_str(&self, address: &Address) -> String {
        AddressEncoder::encode_address_as_string(self.network_prefix, address)
    }

    /// encode address as Base58 encoded string
    pub fn encode_address_as_bytes(network_prefix: NetworkPrefix, address: &Address) -> Vec<u8> {
        let prefix_byte = network_prefix as u8 + address.address_type_prefix() as u8;
        let mut address_bytes = address.content_bytes();
        let mut bytes = vec![prefix_byte];
        bytes.append(&mut address_bytes);
        let mut calculated_checksum = AddressEncoder::calc_checksum(&bytes[..]).to_vec();
        bytes.append(&mut calculated_checksum);
        bytes
    }

    /// encode address as Base58 encoded string
    pub fn encode_address_as_string(network_prefix: NetworkPrefix, address: &Address) -> String {
        bs58::encode(AddressEncoder::encode_address_as_bytes(
            network_prefix,
            address,
        ))
        .into_string()
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
pub(crate) mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Address {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let non_parseable_tree = "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301";
            prop_oneof![
                any::<ErgoTree>().prop_map(|t| match ProveDlog::try_from(t.clone()) {
                    Ok(dlog) => Address::P2Pk(dlog),
                    Err(_) => Address::P2S(t.sigma_serialize_bytes().unwrap()),
                }),
                any::<ErgoTree>().prop_map(|t| {
                    let bytes = t.sigma_serialize_bytes().unwrap();
                    let address: [u8; 24] = blake2b256_hash(&bytes)[0..24].try_into().unwrap();
                    Address::P2SH(address)
                }),
                Just(Address::P2S(base16::decode(non_parseable_tree).unwrap()))
            ]
            .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn str_roundtrip(v in any::<Address>()) {
            let encoder = AddressEncoder::new(NetworkPrefix::Testnet);
            let encoded_addr = encoder.address_to_str(&v);
            let decoded_addr = encoder.parse_address_from_str(&encoded_addr).unwrap();
            prop_assert_eq![decoded_addr, v];
        }

        #[test]
        fn recreate_roundtrip(v in any::<Address>()) {
            let tree = v.script().unwrap();
            let recreated = Address::recreate_from_ergo_tree(&tree).unwrap();
            prop_assert_eq![recreated, v];
        }

        #[test]
        fn doesnt_crash_on_invalid_input(s in "\\w+") {
            let encoder = AddressEncoder::new(NetworkPrefix::Testnet);
            prop_assert![encoder.parse_address_from_str(&s).is_err()];
        }
    }
}

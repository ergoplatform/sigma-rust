use super::digest32;
use crate::{
    ast::Expr,
    serialization::{SerializationError, SigmaSerializable},
    sigma_protocol::{
        dlog_group::EcPoint,
        sigma_boolean::{ProveDlog, SigmaBoolean, SigmaProofOfKnowledgeTree, SigmaProp},
    },
    ErgoTree,
};
use std::{
    convert::{TryFrom, TryInto},
    rc::Rc,
};
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
    P2PK(ProveDlog),
    /// serialized script
    P2S(Vec<u8>),
    // P2SH([u8; 24]),
}

impl Address {
    /// address type prefix (for encoding)
    pub fn address_type_prefix(&self) -> AddressTypePrefix {
        match self {
            Address::P2PK(_) => AddressTypePrefix::P2PK,
            Address::P2S(_) => AddressTypePrefix::Pay2S,
            //Address::P2SH(_) => AddressTypePrefix::P2SH,
        }
    }

    /// byte array
    pub fn content_bytes(&self) -> Vec<u8> {
        match self {
            Address::P2PK(prove_dlog) => prove_dlog.h.sigma_serialise_bytes(),
            Address::P2S(bytes) => bytes.clone(),
        }
    }

    /// script encoded in the address
    pub fn script(&self) -> Result<ErgoTree, SerializationError> {
        match self {
            Address::P2PK(prove_dlog) => Ok(ErgoTree::from(Rc::new(Expr::Const(
                SigmaProp::new(SigmaBoolean::ProofOfKnowledge(
                    SigmaProofOfKnowledgeTree::ProveDlog(prove_dlog.clone()),
                ))
                .into(),
            )))),
            Address::P2S(bytes) => ErgoTree::sigma_parse_bytes(bytes.to_vec()),
        }
    }
}

/// Address types
pub enum AddressTypePrefix {
    /// 0x01 - Pay-to-PublicKey(P2PK) address
    P2PK = 1,
    /// 0x02 - Pay-to-Script-Hash(P2SH)
    Pay2SH = 2,
    /// 0x03 - Pay-to-Script(P2S)
    Pay2S = 3,
}

impl TryFrom<u8> for AddressTypePrefix {
    type Error = AddressEncoderError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == AddressTypePrefix::P2PK as u8 => Ok(AddressTypePrefix::P2PK),
            v if v == AddressTypePrefix::Pay2SH as u8 => Ok(AddressTypePrefix::Pay2SH),
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
    DeserializationFailed(SerializationError),
}

impl From<bs58::decode::Error> for AddressEncoderError {
    fn from(err: bs58::decode::Error) -> Self {
        AddressEncoderError::Base58DecodingError(err.to_string())
    }
}

impl From<SerializationError> for AddressEncoderError {
    fn from(err: SerializationError) -> Self {
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
        let v: Vec<u8> = digest32::blake2b256_hash(bytes)
            .0
            .to_vec()
            .into_iter()
            .take(AddressEncoder::CHECKSUM_LENGTH)
            .collect();
        v.as_slice().try_into().unwrap()
    }

    /// parse address from Base58 encoded string
    pub fn parse_address_from_str(&self, str: &str) -> Result<Address, AddressEncoderError> {
        let bytes = bs58::decode(str).into_vec()?;
        if bytes.len() < AddressEncoder::MIN_ADDRESS_LENGTH {
            return Err(AddressEncoderError::InvalidSize);
        };
        let head_byte = self.check_head_byte(bytes[0])?;
        let (without_checksum, checksum) =
            bytes.split_at(bytes.len() - AddressEncoder::CHECKSUM_LENGTH);
        let calculated_checksum = AddressEncoder::calc_checksum(without_checksum);
        if checksum != calculated_checksum {
            return Err(AddressEncoderError::InvalidChecksum);
        };

        let content_bytes: Vec<u8> = without_checksum[1..].to_vec(); // without head_byte
        let address_type = AddressTypePrefix::try_from(head_byte - self.network_prefix as u8)?;
        Ok(match address_type {
            AddressTypePrefix::P2PK => {
                Address::P2PK(ProveDlog::new(EcPoint::sigma_parse_bytes(content_bytes)?))
            }
            AddressTypePrefix::Pay2S => Address::P2S(content_bytes),
            AddressTypePrefix::Pay2SH => todo!(),
        })
    }

    /// encode address as Base58 encoded string
    pub fn address_to_str(&self, address: &Address) -> String {
        let prefix_byte = self.network_prefix as u8 + address.address_type_prefix() as u8;
        let mut address_bytes = address.content_bytes();
        let mut bytes = vec![prefix_byte];
        bytes.append(&mut address_bytes);
        let mut calculated_checksum = AddressEncoder::calc_checksum(&bytes[..]).to_vec();
        bytes.append(&mut calculated_checksum);
        bs58::encode(bytes).into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for Address {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                any::<ProveDlog>().prop_map(Address::P2PK),
                any::<ErgoTree>().prop_map(|t| Address::P2S(t.sigma_serialise_bytes())),
            ]
            .boxed()
        }
    }

    proptest! {

        #[test]
        fn str_roundtrip(v in any::<Address>()) {
            let encoder = AddressEncoder::new(NetworkPrefix::Testnet);
            let encoded_addr = encoder.address_to_str(&v);
            let decoded_addr = encoder.parse_address_from_str(&encoded_addr).unwrap();
            prop_assert_eq![decoded_addr, v];
        }

        #[test]
        fn doesnt_crash_on_invalid_input(s in "\\w+") {
            let encoder = AddressEncoder::new(NetworkPrefix::Testnet);
            prop_assert![encoder.parse_address_from_str(&s).is_err()];
        }
    }
}

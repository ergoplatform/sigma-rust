//! Block header
use num_bigint::BigInt;

use crate::sigma_protocol::dlog_group;

use super::block_id::BlockId;
use super::digest32::ADDigest;
use super::digest32::Digest32;
use super::votes::Votes;
use super::preheader::PreHeader;

/// Represents data of the block header available in Sigma propositions.
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Header {
    /// Block version, to be increased on every soft and hardfork.
    #[cfg_attr(feature = "json", serde(rename = "version"))]
    pub version: u8,
    /// Bytes representation of ModifierId of this Header
    #[cfg_attr(feature = "json", serde(rename = "id"))]
    pub id: BlockId,
    /// Bytes representation of ModifierId of the parent block
    #[cfg_attr(feature = "json", serde(rename = "parentId"))]
    pub parent_id: BlockId,
    /// Hash of ADProofs for transactions in a block
    #[cfg_attr(feature = "json", serde(rename = "adProofsRoot"))]
    pub ad_proofs_root: Digest32,
    /// AvlTree of a state after block application
    #[cfg_attr(feature = "json", serde(rename = "stateRoot"))]
    pub state_root: ADDigest,
    /// Root hash (for a Merkle tree) of transactions in a block.
    #[cfg_attr(feature = "json", serde(rename = "transactionsRoot"))]
    pub transaction_root: Digest32,
    /// Timestamp of a block in ms from UNIX epoch
    #[cfg_attr(feature = "json", serde(rename = "timestamp"))]
    pub timestamp: u64,
    /// Current difficulty in a compressed view.
    #[cfg_attr(feature = "json", serde(rename = "nBits"))]
    pub n_bits: u64,
    /// Block height
    #[cfg_attr(feature = "json", serde(rename = "height"))]
    pub height: u32,
    /// Root hash of extension section
    #[cfg_attr(feature = "json", serde(rename = "extensionHash"))]
    pub extension_root: Digest32,
    /// Public key of miner. Part of Autolykos solution.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    pub miner_pk: Box<dlog_group::EcPoint>,
    /// One-time public key. Prevents revealing of miners secret.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    pub pow_onetime_pk: Box<dlog_group::EcPoint>,
    /// nonce
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    pub nonce: Vec<u8>,
    /// Distance between pseudo-random number, corresponding to nonce `nonce` and a secret,
    /// corresponding to `miner_pk`. The lower `pow_distance` is, the harder it was to find this solution.
    #[cfg_attr(feature = "json", serde(skip_serializing, skip_deserializing))]
    pub pow_distance: BigInt,
    /// Miner votes for changing system parameters.
    /// 3 bytes in accordance to Scala implementation, but will use `Vec` until further improvements
    #[cfg_attr(feature = "json", serde(rename = "votes"))]
    pub votes: Votes,
}

impl Header {
    /// Dummy instance intended for tests where actual values are not used
    pub fn dummy() -> Self {
        let empty_digest = Digest32::zero();
        Header {
            version: 1,
            id: BlockId(empty_digest.clone()),
            parent_id: BlockId(empty_digest.clone()),
            ad_proofs_root: empty_digest.clone(),
            state_root: ADDigest::zero(),
            transaction_root: empty_digest.clone(),
            timestamp: 0,
            n_bits: 0,
            height: 0,
            extension_root: empty_digest,
            miner_pk: dlog_group::generator().into(),
            pow_onetime_pk: dlog_group::generator().into(),
            nonce: Vec::new(),
            pow_distance: BigInt::default(),
            votes: Votes([0u8; 3]),
        }
    }
}

// TODO: [sab] refactor when implementing this https://github.com/ergoplatform/sigma-rust/issues/373
impl From<Header> for PreHeader {
    fn from(bh: Header) -> Self {
        PreHeader {
            version: bh.version,
            parent_id: bh.parent_id.0.into(),
            timestamp: bh.timestamp,
            n_bits: bh.n_bits,
            height: bh.height,
            miner_pk: dlog_group::identity().into(), // TODO: get from bh.powSolution when its implemented
            votes: bh.votes.into(),
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use num_bigint::BigInt;
    use proptest::array::{uniform3, uniform32};
    use proptest::prelude::*;

    use crate::chain::digest32::ADDigest;
    use crate::chain::digest32::Digest;
    use crate::sigma_protocol::dlog_group::EcPoint;

    use super::{BlockId, Header, Votes};

    impl Arbitrary for Header {
        type Parameters = ();
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                // Timestamps between 2000-2050
                946_674_000_000..2_500_400_300_000u64,
                any::<u64>(),
                0..1_000_000u32,
                any::<Box<EcPoint>>(),
                any::<Box<EcPoint>>(),
                uniform3(1u8..),
            )
                .prop_map(
                    |(
                        id,
                        parent_id,
                        ad_proofs_root,
                        transaction_root,
                        extension_root,
                        timestamp,
                        n_bits,
                        height,
                        miner_pk,
                        pow_onetime_pk,
                        votes,
                    )| {
                        let id = BlockId(Digest(id.into()));
                        let parent_id = BlockId(Digest(parent_id.into()));
                        let ad_proofs_root = Digest(ad_proofs_root.into());
                        let transaction_root = Digest(transaction_root.into());
                        let extension_root = Digest(extension_root.into());
                        let votes = Votes(votes);
                        Self {
                            version: 1,
                            id,
                            parent_id,
                            ad_proofs_root,
                            state_root: ADDigest::zero(),
                            transaction_root,
                            timestamp,
                            n_bits,
                            height,
                            extension_root,
                            miner_pk,
                            pow_onetime_pk,
                            nonce: Vec::new(),
                            pow_distance: BigInt::default(),
                            votes,
                        }
                    },
                )
                .boxed()
        }

        type Strategy = BoxedStrategy<Header>;
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::chain::header::Header;

    #[test]
    fn parse_block_header() {
        let json = r#"{
            "extensionId": "d16f25b14457186df4c5f6355579cc769261ce1aebc8209949ca6feadbac5a3f",
            "difficulty": "626412390187008",
            "votes": "040000",
            "timestamp": 1618929697400,
            "size": 221,
            "stateRoot": "8ad868627ea4f7de6e2a2fe3f98fafe57f914e0f2ef3331c006def36c697f92713",
            "height": 471746,
            "nBits": 117586360,
            "version": 2,
            "id": "4caa17e62fe66ba7bd69597afdc996ae35b1ff12e0ba90c22ff288a4de10e91b",
            "adProofsRoot": "d882aaf42e0a95eb95fcce5c3705adf758e591532f733efe790ac3c404730c39",
            "transactionsRoot": "63eaa9aff76a1de3d71c81e4b2d92e8d97ae572a8e9ab9e66599ed0912dd2f8b",
            "extensionHash": "3f91f3c680beb26615fdec251aee3f81aaf5a02740806c167c0f3c929471df44",
            "powSolutions": {
              "pk": "02b3a06d6eaa8671431ba1db4dd427a77f75a5c2acbd71bfb725d38adc2b55f669",
              "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
              "n": "5939ecfee6b0d7f4",
              "d": 0
            },
            "adProofsId": "86eaa41f328bee598e33e52c9e515952ad3b7874102f762847f17318a776a7ae",
            "transactionsId": "ac80245714f25aa2fafe5494ad02a26d46e7955b8f5709f3659f1b9440797b3e",
            "parentId": "6481752bace5fa5acba5d5ef7124d48826664742d46c974c98a2d60ace229a34"
        }"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header.height, 471746);
    }

    #[test]
    fn parse_block_header_explorer_v1() {
        // see https://api.ergoplatform.com/api/v1/blocks/de68a9cd727510d01eae3146f862261661f3bebdfd3c45c19d431b2ae81fb4b6
        let json = r#"{
            "extensionId": "d16f25b14457186df4c5f6355579cc769261ce1aebc8209949ca6feadbac5a3f",
            "difficulty": "626412390187008",
            "votes": [4,0,0],
            "timestamp": 1618929697400,
            "size": 221,
            "stateRoot": "8ad868627ea4f7de6e2a2fe3f98fafe57f914e0f2ef3331c006def36c697f92713",
            "height": 471746,
            "nBits": 117586360,
            "version": 2,
            "id": "4caa17e62fe66ba7bd69597afdc996ae35b1ff12e0ba90c22ff288a4de10e91b",
            "adProofsRoot": "d882aaf42e0a95eb95fcce5c3705adf758e591532f733efe790ac3c404730c39",
            "transactionsRoot": "63eaa9aff76a1de3d71c81e4b2d92e8d97ae572a8e9ab9e66599ed0912dd2f8b",
            "extensionHash": "3f91f3c680beb26615fdec251aee3f81aaf5a02740806c167c0f3c929471df44",
            "powSolutions": {
              "pk": "02b3a06d6eaa8671431ba1db4dd427a77f75a5c2acbd71bfb725d38adc2b55f669",
              "w": "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
              "n": "5939ecfee6b0d7f4",
              "d": 0
            },
            "adProofsId": "86eaa41f328bee598e33e52c9e515952ad3b7874102f762847f17318a776a7ae",
            "transactionsId": "ac80245714f25aa2fafe5494ad02a26d46e7955b8f5709f3659f1b9440797b3e",
            "parentId": "6481752bace5fa5acba5d5ef7124d48826664742d46c974c98a2d60ace229a34"
        }"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header.height, 471746);
    }
}

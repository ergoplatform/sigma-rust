//! Block header
use ergo_chain_types::ADDigest;
use ergo_chain_types::BlockId;
use ergo_chain_types::Digest32;
use num_bigint::BigInt;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use sigma_ser::{
    ScorexParsingError, ScorexSerializable, ScorexSerializationError, ScorexSerializeResult,
};
use sigma_util::hash::blake2b256_hash;
use std::io::Write;

use crate::serialization::sigma_byte_writer::SigmaByteWriter;
use crate::sigma_protocol::dlog_group::{self, EcPoint};

use super::preheader::PreHeader;
use super::votes::Votes;

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
    /// Solution for an Autolykos PoW puzzle
    #[cfg_attr(feature = "json", serde(rename = "powSolutions"))]
    pub autolykos_solution: AutolykosSolution,
    /// Miner votes for changing system parameters.
    /// 3 bytes in accordance to Scala implementation, but will use `Vec` until further improvements
    #[cfg_attr(feature = "json", serde(rename = "votes"))]
    pub votes: Votes,
}

impl Header {
    /// Used in nipowpow
    pub fn serialize_without_pow(&self) -> Result<Vec<u8>, ScorexSerializationError> {
        use byteorder::{BigEndian, WriteBytesExt};
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        w.put_u8(self.version)?;
        self.parent_id.0.scorex_serialize(&mut w)?;
        self.ad_proofs_root.scorex_serialize(&mut w)?;
        self.transaction_root.scorex_serialize(&mut w)?;
        self.state_root.scorex_serialize(&mut w)?;
        w.put_u64(self.timestamp)?;
        self.extension_root.scorex_serialize(&mut w)?;

        // n_bits needs to be serialized in big-endian format. Note that it actually fits in a
        // `u32`.
        let mut n_bits_writer = vec![];
        #[allow(clippy::unwrap_used)]
        n_bits_writer
            .write_u32::<BigEndian>(self.n_bits as u32)
            .unwrap();
        w.write_all(&n_bits_writer)?;

        w.put_u32(self.height)?;
        w.write_all(&self.votes.0)?;

        // For block version >= 2, this new byte encodes length of possible new fields.
        // Set to 0 for now, so no new fields.
        if self.version > 1 {
            w.put_i8(0)?;
        }
        Ok(data)
    }
}

impl ScorexSerializable for Header {
    fn scorex_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> ScorexSerializeResult {
        let bytes = self.serialize_without_pow()?;
        w.write_all(&bytes)?;

        // Serialize `AutolykosSolution`
        self.autolykos_solution.serialize_bytes(self.version, w)?;
        Ok(())
    }

    fn scorex_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, ScorexParsingError> {
        let version = r.get_u8()?;
        let parent_id = BlockId(Digest32::scorex_parse(r)?);
        let ad_proofs_root = Digest32::scorex_parse(r)?;
        let transaction_root = Digest32::scorex_parse(r)?;
        let state_root = ADDigest::scorex_parse(r)?;
        let timestamp = r.get_u64()?;
        let extension_root = Digest32::scorex_parse(r)?;
        let mut n_bits_buf = [0u8, 0, 0, 0];
        r.read_exact(&mut n_bits_buf)?;
        let n_bits = {
            use byteorder::{BigEndian, ReadBytesExt};
            let mut reader = std::io::Cursor::new(n_bits_buf);
            #[allow(clippy::unwrap_used)]
            {
                reader.read_u32::<BigEndian>().unwrap() as u64
            }
        };
        let height = r.get_u32()?;
        let mut votes_bytes = [0u8, 0, 0];
        r.read_exact(&mut votes_bytes)?;
        let votes = Votes(votes_bytes);

        // For block version >= 2, a new byte encodes length of possible new fields.  If this byte >
        // 0, we read new fields but do nothing, as semantics of the fields is not known.
        if version > 1 {
            let new_field_size = r.get_u8()?;
            if new_field_size > 0 {
                let mut field_bytes: Vec<u8> =
                    std::iter::repeat(0).take(new_field_size as usize).collect();
                r.read_exact(&mut field_bytes)?;
            }
        }

        // Parse `AutolykosSolution`
        let autolykos_solution = if version == 1 {
            let miner_pk = EcPoint::scorex_parse(r)?.into();
            let pow_onetime_pk = Some(EcPoint::scorex_parse(r)?.into());
            let mut nonce: Vec<u8> = std::iter::repeat(0).take(8).collect();
            r.read_exact(&mut nonce)?;
            let d_bytes_len = r.get_u8()?;
            let mut d_bytes: Vec<u8> = std::iter::repeat(0).take(d_bytes_len as usize).collect();
            r.read_exact(&mut d_bytes)?;
            let pow_distance = Some(BigInt::from_signed_bytes_be(&d_bytes));
            AutolykosSolution {
                miner_pk,
                pow_onetime_pk,
                nonce,
                pow_distance,
            }
        } else {
            // autolykos v2
            let pow_onetime_pk = None;
            let pow_distance = None;
            let miner_pk = EcPoint::scorex_parse(r)?.into();
            let mut nonce: Vec<u8> = std::iter::repeat(0).take(8).collect();
            r.read_exact(&mut nonce)?;
            AutolykosSolution {
                miner_pk,
                pow_onetime_pk,
                nonce,
                pow_distance,
            }
        };

        // The `Header.id` field isn't serialized/deserialized but rather computed as a hash of
        // every other field in `Header`. First we initialize header with dummy id field then
        // compute the hash.
        let mut header = Header {
            version,
            id: BlockId(Digest32::zero()),
            parent_id,
            ad_proofs_root,
            state_root,
            transaction_root,
            timestamp,
            n_bits,
            height,
            extension_root,
            autolykos_solution: autolykos_solution.clone(),
            votes,
        };

        let mut id_bytes = header.serialize_without_pow()?;
        let mut data = Vec::new();
        let mut w = SigmaByteWriter::new(&mut data, None);
        autolykos_solution.serialize_bytes(version, &mut w)?;
        id_bytes.extend(data);
        let id = BlockId(blake2b256_hash(&id_bytes).into());
        header.id = id;
        Ok(header)
    }
}

/// Solution for an Autolykos PoW puzzle. In Autolykos v.1 all the four fields are used, in
/// Autolykos v.2 only `miner_pk` and `nonce` fields are used.
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AutolykosSolution {
    /// Public key of miner. Part of Autolykos solution.
    #[cfg_attr(feature = "json", serde(rename = "pk"))]
    pub miner_pk: Box<dlog_group::EcPoint>,
    /// One-time public key. Prevents revealing of miners secret.
    #[cfg_attr(feature = "json", serde(default, rename = "w"))]
    pub pow_onetime_pk: Option<Box<dlog_group::EcPoint>>,
    /// nonce
    #[cfg_attr(
        feature = "json",
        serde(
            rename = "n",
            serialize_with = "crate::chain::json::autolykos_solution::as_base16_string",
            deserialize_with = "crate::chain::json::autolykos_solution::from_base16_string"
        )
    )]
    pub nonce: Vec<u8>,
    /// Distance between pseudo-random number, corresponding to nonce `nonce` and a secret,
    /// corresponding to `miner_pk`. The lower `pow_distance` is, the harder it was to find this
    /// solution.
    ///
    /// Note: we serialize/deserialize through custom functions since `BigInt`s serde implementation
    /// encodes the sign and absolute-value of the value separately, which is incompatible with the
    /// JSON representation used by Ergo. ASSUMPTION: we assume that `pow_distance` encoded as a
    /// `u64`.
    #[cfg_attr(
        feature = "json",
        serde(
            default,
            rename = "d",
            serialize_with = "crate::chain::json::autolykos_solution::bigint_as_str",
            deserialize_with = "crate::chain::json::autolykos_solution::bigint_from_serde_json_number"
        )
    )]
    pub pow_distance: Option<BigInt>,
}

impl AutolykosSolution {
    /// Serialize instance
    pub fn serialize_bytes<W: WriteSigmaVlqExt>(
        &self,
        version: u8,
        w: &mut W,
    ) -> Result<(), ScorexSerializationError> {
        if version == 1 {
            self.miner_pk.scorex_serialize(w)?;
            self.pow_onetime_pk
                .as_ref()
                .ok_or(ScorexSerializationError::Misc(
                    "pow_onetime_pk must == Some(_) for autolykos v1",
                ))?
                .scorex_serialize(w)?;
            w.write_all(&self.nonce)?;

            let d_bytes = self
                .pow_distance
                .as_ref()
                .ok_or(ScorexSerializationError::Misc(
                    "pow_distance must be == Some(_) for autolykos v1",
                ))?
                .to_signed_bytes_be();
            w.put_u8(d_bytes.len() as u8)?;
            w.write_all(&d_bytes)?;
        } else {
            // Autolykos v2
            self.miner_pk.scorex_serialize(w)?;
            w.write_all(&self.nonce)?;
        }
        Ok(())
    }
}

impl From<Header> for PreHeader {
    fn from(bh: Header) -> Self {
        PreHeader {
            version: bh.version,
            parent_id: bh.parent_id,
            timestamp: bh.timestamp,
            n_bits: bh.n_bits,
            height: bh.height,
            miner_pk: bh.autolykos_solution.miner_pk,
            votes: bh.votes,
        }
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {

    use crate::serialization::sigma_byte_writer::SigmaByteWriter;
    use crate::sigma_protocol::dlog_group::EcPoint;
    use ergo_chain_types::Digest;
    use ergo_chain_types::Digest32;
    use ergo_chain_types::{blake2b256_hash, ADDigest};
    use num_bigint::BigInt;
    use proptest::array::{uniform3, uniform32};
    use proptest::prelude::*;

    use super::{AutolykosSolution, BlockId, Header, Votes};

    impl Arbitrary for Header {
        type Parameters = ();
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                uniform32(1u8..),
                // Timestamps between 2000-2050
                946_674_000_000..2_500_400_300_000u64,
                any::<u32>(), // Note: n_bits must fit in u32
                1_000_000u32..10_000_000u32,
                prop::sample::select(vec![1_u8, 2]),
                any::<Box<AutolykosSolution>>(),
                uniform3(1u8..),
            )
                .prop_map(
                    |(
                        parent_id,
                        ad_proofs_root,
                        transaction_root,
                        extension_root,
                        timestamp,
                        n_bits,
                        height,
                        version,
                        autolykos_solution,
                        votes,
                    )| {
                        let parent_id = BlockId(Digest(parent_id.into()));
                        let ad_proofs_root = Digest(ad_proofs_root.into());
                        let transaction_root = Digest(transaction_root.into());
                        let extension_root = Digest(extension_root.into());
                        let votes = Votes(votes);

                        // The `Header.id` field isn't serialized/deserialized but rather computed
                        // as a hash of every other field in `Header`. First we initialize header
                        // with dummy id field then compute the hash.
                        let mut header = Self {
                            version,
                            id: BlockId(Digest32::zero()),
                            parent_id,
                            ad_proofs_root,
                            state_root: ADDigest::zero(),
                            transaction_root,
                            timestamp,
                            n_bits: n_bits as u64,
                            height,
                            extension_root,
                            autolykos_solution: *autolykos_solution.clone(),
                            votes,
                        };
                        let mut id_bytes = header.serialize_without_pow().unwrap();
                        let mut data = Vec::new();
                        let mut w = SigmaByteWriter::new(&mut data, None);
                        autolykos_solution.serialize_bytes(version, &mut w).unwrap();
                        id_bytes.extend(data);
                        let id = BlockId(blake2b256_hash(&id_bytes));
                        header.id = id;

                        // Manually set the following parameters to `None` for autolykos v2. This is
                        // allowable since serialization/deserialization of the `Header` ignores
                        // these fields for version > 1.
                        if header.version > 1 {
                            header.autolykos_solution.pow_onetime_pk = None;
                            header.autolykos_solution.pow_distance = None;
                        }

                        header
                    },
                )
                .boxed()
        }

        type Strategy = BoxedStrategy<Header>;
    }

    impl Arbitrary for AutolykosSolution {
        type Parameters = ();
        type Strategy = BoxedStrategy<AutolykosSolution>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<Box<EcPoint>>(),
                prop::collection::vec(0_u8.., 8),
                any::<Box<EcPoint>>(),
                any::<u64>(),
            )
                .prop_map(
                    |(miner_pk, nonce, pow_onetime_pk, pow_distance)| AutolykosSolution {
                        miner_pk,
                        nonce,
                        pow_onetime_pk: Some(pow_onetime_pk),
                        pow_distance: Some(BigInt::from(pow_distance)),
                    },
                )
                .boxed()
        }
    }
}

#[allow(clippy::unwrap_used, clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::str::FromStr;

    use num_bigint::BigInt;

    use crate::chain::header::Header;
    use proptest::prelude::*;
    use sigma_ser::scorex_serialize_roundtrip;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn ser_roundtrip(v in any::<Header>()) {
            assert_eq![scorex_serialize_roundtrip(&v), v]
        }
    }

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
              "d": "1234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            },
            "adProofsId": "86eaa41f328bee598e33e52c9e515952ad3b7874102f762847f17318a776a7ae",
            "transactionsId": "ac80245714f25aa2fafe5494ad02a26d46e7955b8f5709f3659f1b9440797b3e",
            "parentId": "6481752bace5fa5acba5d5ef7124d48826664742d46c974c98a2d60ace229a34"
        }"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header.height, 471746);
        assert_eq!(
            header.autolykos_solution.pow_distance,
            Some(BigInt::from_str(
                "1234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap())
        );
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
              "d": 1234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
            },
            "adProofsId": "86eaa41f328bee598e33e52c9e515952ad3b7874102f762847f17318a776a7ae",
            "transactionsId": "ac80245714f25aa2fafe5494ad02a26d46e7955b8f5709f3659f1b9440797b3e",
            "parentId": "6481752bace5fa5acba5d5ef7124d48826664742d46c974c98a2d60ace229a34"
        }"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header.height, 471746);
        assert_eq!(
            header.autolykos_solution.pow_distance,
            Some(BigInt::from_str(
                "1234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap())
        );
    }
}

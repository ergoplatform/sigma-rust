//! Block header
use crate::autolykos_pow_scheme::{
    decode_compact_bits, order_bigint, AutolykosPowScheme, AutolykosPowSchemeError,
};
use crate::{ADDigest, BlockId, Digest32, EcPoint};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core3::io::Write;
use num_bigint::{BigUint, ToBigInt};
use num_traits::Zero;
use sigma_ser::vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};
use sigma_ser::{
    ScorexParsingError, ScorexSerializable, ScorexSerializationError, ScorexSerializeResult,
};
use sigma_util::hash::blake2b256_hash;

use crate::votes::Votes;

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
    pub n_bits: u32,
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
    /// Unparsed bytes that encode new possible fields
    #[cfg_attr(
        feature = "json",
        serde(
            rename = "unparsedBytes",
            default,
            serialize_with = "crate::json::autolykos_solution::as_base16_string",
            deserialize_with = "crate::json::autolykos_solution::from_base16_string"
        )
    )]
    pub unparsed_bytes: Box<[u8]>,
}

impl Header {
    /// Used in nipowpow
    pub fn serialize_without_pow(&self) -> Result<Vec<u8>, ScorexSerializationError> {
        let mut data = Vec::new();
        let mut w = &mut data;
        w.put_u8(self.version)?;
        self.parent_id.0.scorex_serialize(&mut w)?;
        self.ad_proofs_root.scorex_serialize(&mut w)?;
        self.transaction_root.scorex_serialize(&mut w)?;
        self.state_root.scorex_serialize(&mut w)?;
        w.put_u64(self.timestamp)?;
        self.extension_root.scorex_serialize(&mut w)?;

        // n_bits needs to be serialized in big-endian format. Note that it actually fits in a
        // `u32`.
        let n_bits_be = self.n_bits.to_be_bytes();
        w.write_all(&n_bits_be)?;

        w.put_u32(self.height)?;
        w.write_all(&self.votes.0)?;

        // For block version >= 2, this new byte encodes length of possible new fields.
        // Set to 0 for now, so no new fields.
        if self.version > 1 {
            w.put_u8(self.unparsed_bytes.len().try_into()?)?;
            w.write_all(&self.unparsed_bytes)?;
        }
        Ok(data)
    }
    /// Check that proof of work was valid for header. Only Autolykos2 is supported
    /// Returns [`AutolykosPowSchemeError::OutOfBounds`] if the decoded difficulty is zero.
    pub fn check_pow(&self) -> Result<bool, AutolykosPowSchemeError> {
        if self.version != 1 {
            let hit = AutolykosPowScheme::default().pow_hit(self)?;
            let difficulty = decode_compact_bits(self.n_bits);
            if difficulty.is_zero() {
                return Err(AutolykosPowSchemeError::OutOfBounds);
            }
            let target = order_bigint() / difficulty;
            #[allow(clippy::unwrap_used)] // unsigned -> signed conversion never fails
            Ok(hit.to_bigint().unwrap() < target)
        } else {
            Err(AutolykosPowSchemeError::Unsupported)
        }
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
        r.get_bytes_into(&mut n_bits_buf)?;
        let n_bits = u32::from_be_bytes(n_bits_buf);
        let height = r.get_u32()?;
        let mut votes_bytes = [0u8, 0, 0];
        r.get_bytes_into(&mut votes_bytes)?;
        let votes = Votes(votes_bytes);

        // For block version >= 2, a new byte encodes length of possible new fields.  If this byte >
        // 0, we read new fields but do nothing, as semantics of the fields is not known.
        // As in the node, new fields are read only for block version >= 5; for versions 2..=4 the
        // length byte is read and the bytes after it are parsed as the PoW solution.
        let unparsed_bytes: Box<[u8]> = if version > 1 {
            let new_field_size = r.get_u8()?;
            if new_field_size > 0 && version > 4 {
                let mut field_bytes: Vec<u8> = vec![0; new_field_size as usize];
                r.get_bytes_into(&mut field_bytes)?;
                field_bytes.into()
            } else {
                Box::new([])
            }
        } else {
            Box::new([])
        };

        // Parse `AutolykosSolution`
        let autolykos_solution = if version == 1 {
            let miner_pk = EcPoint::scorex_parse(r)?.into();
            let pow_onetime_pk = Some(EcPoint::scorex_parse(r)?.into());
            let mut nonce: Vec<u8> = vec![0; 8];
            r.get_bytes_into(&mut nonce)?;
            let d_bytes_len = r.get_u8()?;
            let mut d_bytes: Vec<u8> = vec![0; d_bytes_len as usize];
            r.get_bytes_into(&mut d_bytes)?;
            let pow_distance = Some(BigUint::from_bytes_be(&d_bytes));
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
            let mut nonce: Vec<u8> = vec![0; 8];
            r.get_bytes_into(&mut nonce)?;
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
            unparsed_bytes,
        };

        let mut id_bytes = header.serialize_without_pow()?;
        let mut data = Vec::new();
        autolykos_solution.serialize_bytes(version, &mut data)?;
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
    pub miner_pk: Box<EcPoint>,
    /// One-time public key. Prevents revealing of miners secret.
    #[cfg_attr(feature = "json", serde(default, rename = "w"))]
    pub pow_onetime_pk: Option<Box<EcPoint>>,
    /// nonce
    #[cfg_attr(
        feature = "json",
        serde(
            rename = "n",
            serialize_with = "crate::json::autolykos_solution::as_base16_string",
            deserialize_with = "crate::json::autolykos_solution::from_base16_string"
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
            serialize_with = "crate::json::autolykos_solution::bigint_as_str",
            deserialize_with = "crate::json::autolykos_solution::bigint_from_serde_json_number"
        )
    )]
    pub pow_distance: Option<BigUint>,
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
                .to_bytes_be();
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

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {

    use crate::*;
    use num_bigint::BigUint;
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
                prop::sample::select(vec![1_u8, 2, 3, 4, 5]),
                any::<Box<AutolykosSolution>>(),
                uniform3(1u8..),
                proptest::collection::vec(any::<u8>(), 0..=255),
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
                        unparsed_bytes,
                    )| {
                        let parent_id = BlockId(Digest(parent_id));
                        let ad_proofs_root = Digest(ad_proofs_root);
                        let transaction_root = Digest(transaction_root);
                        let extension_root = Digest(extension_root);
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
                            n_bits,
                            height,
                            extension_root,
                            autolykos_solution: *autolykos_solution.clone(),
                            votes,
                            // new fields are parsed only for block version >= 5
                            unparsed_bytes: if version > 4 {
                                unparsed_bytes.into()
                            } else {
                                Box::new([])
                            },
                        };
                        let mut id_bytes = header.serialize_without_pow().unwrap();
                        let mut data = Vec::new();
                        let mut w = &mut data;
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
                        pow_distance: Some(BigUint::from(pow_distance)),
                    },
                )
                .boxed()
        }
    }
}

#[cfg(test)]
mod pow_boundary_tests {
    use super::*;

    fn header(n_bits: u32) -> Header {
        Header {
            version: 2,
            id: BlockId(Digest32::zero()),
            parent_id: BlockId(Digest32::zero()),
            ad_proofs_root: Digest32::zero(),
            state_root: ADDigest::zero(),
            transaction_root: Digest32::zero(),
            timestamp: 0,
            n_bits,
            height: 1,
            extension_root: Digest32::zero(),
            autolykos_solution: AutolykosSolution {
                miner_pk: Box::new(crate::ec_point::generator()),
                pow_onetime_pk: None,
                nonce: vec![0; 8],
                pow_distance: None,
            },
            votes: Votes([0; 3]),
            unparsed_bytes: Box::new([]),
        }
    }

    #[test]
    fn check_pow_zero_difficulty_returns_error() {
        assert_eq!(
            header(0).check_pow(),
            Err(AutolykosPowSchemeError::OutOfBounds)
        );
    }

    #[test]
    fn check_pow_truncated_zero_difficulty_returns_error() {
        assert_eq!(
            header(0x01003456).check_pow(),
            Err(AutolykosPowSchemeError::OutOfBounds)
        );
    }

    #[test]
    fn check_pow_signed_zero_difficulty_returns_error() {
        assert_eq!(
            header(0x03800000).check_pow(),
            Err(AutolykosPowSchemeError::OutOfBounds)
        );
    }

    #[test]
    fn check_pow_nonzero_compact_boundaries_preserved() {
        assert_eq!(header(0x01010000).check_pow(), Ok(true));
        assert_eq!(header(0x03000001).check_pow(), Ok(true));
        assert_eq!(header(0x01810000).check_pow(), Ok(false));
        assert_eq!(header(0xff7fffff).check_pow(), Ok(false));
        let mut unsupported = header(0);
        unsupported.version = 1;
        assert_eq!(
            unsupported.check_pow(),
            Err(AutolykosPowSchemeError::Unsupported)
        );
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod new_fields_tests {
    use super::*;

    // A version 2 header, 223 bytes. The new fields length byte is at offset 181 (here 0), the
    // miner pk at [182, 215) and the nonce at [215, 223).
    const HEADER_V2: &str = "02ac2101807f0000ca01ff0119db227f202201007f62000177a080005d440896d05d3f80dcff7f5e7f59007294c180808d0158d1ff6ba10000f901c7f0ef87dcfff17fffacb6ff7f7f1180d2ff7f1e24ffffe1ff937f807f0797b9ff6ebdae007e5c8c00b8403d3701557181c8df800001b6d5009e2201c6ff807d71808c00019780f087adb3fcdbc0b3441480887f80007f4b01cf7f013ff1ffff564a0000b9a54f00770e807f41ff88c00240000080c0250000000003bedaee069ff4829500b3c07c4d5fe6b3ea3d3bf76c5c28c1d4dcdb1bed0ade0c0000000000003105";

    /// `HEADER_V2` with the given version, a new fields length of 1 and one new field byte 0x00.
    fn with_one_new_field(version: u8) -> Vec<u8> {
        let h = base16::decode(HEADER_V2).unwrap();
        let mut bytes = vec![version];
        bytes.extend_from_slice(&h[1..181]);
        bytes.extend_from_slice(&[0x01, 0x00]);
        bytes.extend_from_slice(&h[182..]);
        bytes
    }

    // Expected values below are from the node's HeaderSerializer (v6.0.7) on the same bytes.
    #[test]
    fn header_v2_v4_new_fields_are_not_consumed() {
        let node_ids = [
            "2ee8c6bc5702827620121af58fe2be6b9b22a3a6f9571b62cfc48c1fd6d80b01",
            "f9522b6013247b1eba26ea592f739bf31d43e2ad0807ccdd3e41559a3ad83646",
            "520db26b902238ad0bf32ea76cf5a7f02a7cb8e2f81fa259293676532cd5b8ff",
        ];
        for (version, node_id) in (2..=4).zip(node_ids) {
            let bytes = with_one_new_field(version);
            let h = Header::scorex_parse_bytes(&bytes).unwrap();
            assert!(h.unparsed_bytes.is_empty());
            // the new field byte 0x00 is read as the first byte of the miner pk
            assert_eq!(*h.autolykos_solution.miner_pk, crate::ec_point::identity());
            assert_eq!(
                base16::encode_lower(&h.autolykos_solution.nonce),
                "0c00000000000031"
            );
            // the node writes the length byte as 0, the pk as 33 zero bytes, then the nonce
            let mut node_bytes = bytes[..181].to_vec();
            node_bytes.extend_from_slice(&[0; 34]);
            node_bytes.extend_from_slice(&bytes[215..223]);
            let ser = h.scorex_serialize_bytes().unwrap();
            assert_eq!(ser, node_bytes);
            assert_eq!(base16::encode_lower(&h.id.0 .0), node_id);
            assert_eq!(h.id.0 .0, *blake2b256_hash(&ser));
            assert_eq!(Header::scorex_parse_bytes(&ser).unwrap(), h);
        }
    }

    #[test]
    fn header_v5_new_fields_are_consumed() {
        let bytes = with_one_new_field(5);
        let h = Header::scorex_parse_bytes(&bytes).unwrap();
        assert_eq!(&*h.unparsed_bytes, &[0x00]);
        assert_eq!(
            base16::encode_lower(&h.autolykos_solution.nonce),
            "0000000000003105"
        );
        assert_eq!(h.scorex_serialize_bytes().unwrap(), bytes);
        assert_eq!(
            base16::encode_lower(&h.id.0 .0),
            "48e1f80d04ef8f4c096d37bdff75c04370998c8ebf965c859672ff3b587eb4c7"
        );
    }
}

#[allow(clippy::unwrap_used, clippy::panic)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use std::str::FromStr;

    use num_bigint::BigUint;

    use crate::header::Header;
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
        assert!(header.check_pow().unwrap());
        assert_eq!(
            header.autolykos_solution.pow_distance,
            Some(BigUint::from_str(
                "1234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap())
        );
    }

    #[test]
    fn parse_block_header_explorer_v1() {
        // see https://api.ergoplatform.com/api/v1/blocks/41c73753452a292442799bd884fbcc2a9b0f62d4cff7ad02ccd3dbe65791c908
        let json = r#"{
          "extensionId": "0a91aa00954c218cd11e10230d781a935dd6e53a8eab3a1abcf69fbaf7cd2b34",
          "difficulty": "185435213004800",
          "votes": "000000",
          "timestamp": 1562027226367,
          "size": 279,
          "unparsedBytes": "",
          "stateRoot": "144c15900826f6e2aac70cb50e541215b337d0d1674da6b491499944e686b41b0e",
          "height": 3132,
          "nBits": 117483687,
          "version": 1,
          "id": "41c73753452a292442799bd884fbcc2a9b0f62d4cff7ad02ccd3dbe65791c908",
          "adProofsRoot": "c9d58eacf6108c9a166b0b76020e3323c6c2ccec5ec8f905ea46f5bcc58aac80",
          "transactionsRoot": "01bf55fd587291172f458232a7f58b4b29469d72b8e304aafd68401f915b0c36",
          "extensionHash": "ccb136ffd50a16f50a499e1c33d8ae1e8426bdc70b13a4d82275d057be2d04a7",
          "powSolutions": {
            "pk": "02ff03f4b981c59ccd5185fddcd949b8f5697341e60d808d2be0e3e09d2ec78bf4",
            "w": "037427400e5292a177dc242631f78ab322b7845ad2b8491b016b7c36407c6a6d76",
            "n": "0000667700008481",
            "d": 410958177852074551025494081160156537946251159549691138805256284
          },
          "adProofsId": "d0179a444574258d44cfc5bd33d4c8cc03170d29ceb70bb442bd6659f1131a86",
          "transactionsId": "83fa4252d56faa259a732967596cbdc52d6793ae3918cd0e24cc10653450068a",
          "parentId": "150290bbaf91ccd4dcf307cb9a5113eed67e12694ec9be277e8fa55fb5ebf6ac"
        }"#;
        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(scorex_serialize_roundtrip(&header), header);
        assert_eq!(header.height, 3132);
        assert_eq!(
            header.autolykos_solution.pow_distance,
            Some(
                BigUint::from_str(
                    "410958177852074551025494081160156537946251159549691138805256284"
                )
                .unwrap()
            )
        );
    }
}

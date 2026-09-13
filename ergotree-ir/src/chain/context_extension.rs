//! ContextExtension type
use crate::mir::constant::Constant;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use alloc::string::String;
use core::convert::TryFrom;
use core::fmt;
use core::hash::BuildHasher;
use thiserror::Error;

use super::IndexMap;

/// User-defined variables to be put into context
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    feature = "json",
    derive(serde::Deserialize),
    serde(try_from = "IndexMap<String, String>")
)]
pub struct ContextExtension {
    /// key-value pairs of variable id and it's value
    pub values: IndexMap<u8, Constant>,
}

impl ContextExtension {
    /// Returns an empty ContextExtension
    pub fn empty() -> Self {
        Self {
            values: IndexMap::with_hasher(Default::default()),
        }
    }
}

impl fmt::Display for ContextExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.values.iter()).finish()
    }
}

/// Scala 2.12 immutable.HashMap hash improvement function.
/// Used to predict the HAMT (Hash Array Mapped Trie) iteration order
/// that the Ergo node (Scala 2.12) uses for ContextExtension serialization.
///
/// The Ergo node's ContextExtension uses `scala.collection.immutable.Map` which,
/// for 5+ entries, becomes a HashMap with hash-based iteration order. This order
/// differs from sigma-rust's BTreeMap/IndexMap sorted order, causing bytes_to_sign
/// divergence and transaction rejection.
///
/// See: <https://github.com/scala/scala/blob/v2.12.20/src/library/scala/collection/immutable/HashMap.scala>
/// See: <https://github.com/ergoplatform/sigma-rust/issues/763>
fn scala_212_improve(hc: i32) -> i32 {
    let mut h: i32 = hc.wrapping_add(!(hc.wrapping_shl(9)));
    h = h ^ (((h as u32) >> 14) as i32);
    h = h.wrapping_add(h.wrapping_shl(4));
    h ^ (((h as u32) >> 10) as i32)
}

/// Compute a sort key that matches Scala 2.12 HashMap's HAMT iteration order.
/// The HAMT uses 5 bits per level from the improved hash, iterating slots 0-31
/// at each level. The sort key encodes levels from outermost (most significant)
/// to innermost (least significant).
fn scala_212_hamt_sort_key(key: u8) -> u64 {
    let hash = scala_212_improve(key as i32) as u32;
    let mut sort_key: u64 = 0;
    for level in 0..7 {
        sort_key <<= 5;
        sort_key |= ((hash >> (level * 5)) & 0x1f) as u64;
    }
    sort_key
}

impl SigmaSerializable for ContextExtension {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.values.len() as u8)?;
        if self.values.len() >= 5 {
            // For 5+ entries, Scala 2.12 uses HashMap which iterates in HAMT order
            // (based on hash of keys). We must match this order for bytes_to_sign
            // compatibility with the Ergo node.
            // See: https://github.com/ergoplatform/sigma-rust/issues/763
            let mut entries: alloc::vec::Vec<_> = self.values.iter().collect();
            entries.sort_by_key(|(&idx, _)| scala_212_hamt_sort_key(idx));
            for (&idx, c) in entries {
                w.put_u8(idx)?;
                c.sigma_serialize(w)?;
            }
        } else {
            // For 1-4 entries, Scala uses Map1-Map4 which preserves insertion order.
            // IndexMap also preserves insertion order, so they match.
            self.values.iter().try_for_each(|(idx, c)| {
                w.put_u8(*idx)?;
                c.sigma_serialize(w)
            })?;
        }
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let values_count = r.get_u8()?;
        let mut values: IndexMap<u8, Constant> =
            IndexMap::with_capacity_and_hasher(values_count as usize, Default::default());
        for _ in 0..values_count {
            let idx = r.get_u8()?;
            let value = Constant::sigma_parse(r)?;
            value.tpe.check_v6_type()?;
            values.insert(idx, value);
        }
        Ok(ContextExtension { values })
    }
}

/// Error parsing Constant from base16-encoded string
#[derive(Error, Eq, PartialEq, Debug, Clone)]
#[error("Error parsing constant: {0}")]
pub struct ConstantParsingError(pub String);

// for JSON encoding in ergo-lib
impl<H: BuildHasher> TryFrom<indexmap::IndexMap<String, String, H>> for ContextExtension {
    type Error = ConstantParsingError;
    fn try_from(values_str: indexmap::IndexMap<String, String, H>) -> Result<Self, Self::Error> {
        let values = values_str.iter().try_fold(
            IndexMap::with_capacity_and_hasher(values_str.len(), Default::default()),
            |mut acc, pair| {
                let idx: u8 = pair.0.parse().map_err(|_| {
                    ConstantParsingError(format!("cannot parse index from {0:?}", pair.0))
                })?;
                let constant_bytes = base16::decode(pair.1).map_err(|_| {
                    ConstantParsingError(format!(
                        "cannot decode base16 constant bytes from {0:?}",
                        pair.1
                    ))
                })?;
                acc.insert(
                    idx,
                    Constant::sigma_parse_bytes(&constant_bytes).map_err(|_| {
                        ConstantParsingError(format!(
                            "cannot deserialize constant bytes from {0:?}",
                            pair.1
                        ))
                    })?,
                );
                Ok(acc)
            },
        )?;
        Ok(ContextExtension { values })
    }
}

#[cfg(feature = "json")]
impl serde::Serialize for ContextExtension {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(
                &format!("{}", k),
                &base16::encode_lower(&v.sigma_serialize_bytes().map_err(Error::custom)?),
            )?;
        }
        map.end()
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use super::*;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for ContextExtension {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(
                any::<Constant>().prop_filter(
                    "Filter out types that can't be serialized in ContextExtension",
                    |c| c.tpe.check_v6_type().is_ok(),
                ),
                0..10,
            )
            .prop_map(|constants| {
                let pairs = constants
                    .into_iter()
                    .enumerate()
                    .map(|(idx, c)| (idx as u8, c))
                    .collect();
                Self { values: pairs }
            })
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{serialization::sigma_serialize_roundtrip, unsignedbigint256::UnsignedBigInt};
    use proptest::prelude::*;

    #[test]
    #[should_panic]
    fn test_v6_type_reject() {
        let mut extension = ContextExtension::empty();
        extension
            .values
            .insert(0, Constant::from(UnsignedBigInt::from(1u32)));
        sigma_serialize_roundtrip(&extension);
    }

    proptest! {
        #[test]
        fn ser_roundtrip(v in any::<ContextExtension>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
    #[test]
    fn test_scala_212_improve() {
        // Verify that the improve function produces distinct hashes and that
        // the lowest 5 bits (HAMT level-0 slot) match the empirically observed
        // Ergo node iteration order for keys 0-5: [0, 5, 1, 2, 3, 4].
        // Level-0 slots: key→slot: 0→0, 1→7, 2→14, 3→20, 4→29, 5→1
        assert_eq!((scala_212_improve(0) as u32) & 0x1f, 0);
        assert_eq!((scala_212_improve(1) as u32) & 0x1f, 7);
        assert_eq!((scala_212_improve(2) as u32) & 0x1f, 14);
        assert_eq!((scala_212_improve(3) as u32) & 0x1f, 20);
        assert_eq!((scala_212_improve(4) as u32) & 0x1f, 29);
        assert_eq!((scala_212_improve(5) as u32) & 0x1f, 1);
    }

    #[test]
    fn test_hamt_sort_order_6_entries() {
        // For keys {0,1,2,3,4,5}, the Scala 2.12 HashMap HAMT iterates in order
        // [0, 5, 1, 2, 3, 4] due to the improve hash function's slot assignments.
        // This was verified empirically against the Ergo node.
        let mut keys: Vec<u8> = vec![0, 1, 2, 3, 4, 5];
        keys.sort_by_key(|&k| scala_212_hamt_sort_key(k));
        assert_eq!(keys, vec![0, 5, 1, 2, 3, 4]);
    }

    #[test]
    fn test_serialize_order_5plus_entries() {
        // Verify that serialization of 6-entry ContextExtension produces entries
        // in Scala 2.12 HAMT iteration order, not insertion/sorted order.
        use crate::serialization::SigmaSerializable;
        let mut ext = ContextExtension::empty();
        for i in 0..6u8 {
            ext.values.insert(i, Constant::from(i as i32));
        }
        let bytes = ext.sigma_serialize_bytes().unwrap();
        // bytes[0] = count (6)
        assert_eq!(bytes[0], 6);
        // After count, each entry is: key_byte, serialized_constant
        // Extract just the key bytes (every entry is key + 2 bytes for SInt constant)
        let mut keys = Vec::new();
        let mut pos = 1;
        while pos < bytes.len() {
            keys.push(bytes[pos]);
            // SInt constants serialize as type_byte + vlq_value (2 bytes for small ints)
            let c = Constant::sigma_parse_bytes(&bytes[pos + 1..]).unwrap();
            pos += 1 + c.sigma_serialize_bytes().unwrap().len();
        }
        assert_eq!(keys, vec![0, 5, 1, 2, 3, 4]);
    }

    #[test]
    fn test_serialize_order_4_entries_unchanged() {
        // Verify that serialization of 4-entry ContextExtension preserves
        // insertion order (Scala Map1-Map4 behavior).
        use crate::serialization::SigmaSerializable;
        let mut ext = ContextExtension::empty();
        for i in 0..4u8 {
            ext.values.insert(i, Constant::from(i as i32));
        }
        let bytes = ext.sigma_serialize_bytes().unwrap();
        assert_eq!(bytes[0], 4);
        let mut keys = Vec::new();
        let mut pos = 1;
        while pos < bytes.len() {
            keys.push(bytes[pos]);
            let c = Constant::sigma_parse_bytes(&bytes[pos + 1..]).unwrap();
            pos += 1 + c.sigma_serialize_bytes().unwrap().len();
        }
        // 4 entries: insertion order preserved
        assert_eq!(keys, vec![0, 1, 2, 3]);
    }

    #[cfg(feature = "json")]
    mod json {
        use super::*;
        #[test]
        fn parse_empty_context_extension() {
            let c: ContextExtension = serde_json::from_str("{}").unwrap();
            assert_eq!(c, ContextExtension::empty());
        }

        #[test]
        fn parse_context_extension() {
            let json = r#"
            {"1" :"05b0b5cad8e6dbaef44a", "3":"048ce5d4e505"}
            "#;
            let c: ContextExtension = serde_json::from_str(json).unwrap();
            assert_eq!(c.values.len(), 2);
            assert!(c.values.get(&1u8).is_some());
            assert!(c.values.get(&3u8).is_some());
        }
    }
}

use ergo_chain_types::ADDigest;
use sigma_ser::ScorexSerializable;

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SigmaParsingError,
    SigmaSerializable, SigmaSerializeResult,
};

#[derive(PartialEq, Eq, Debug, Clone)]
/// AVL tree flags
pub struct AvlTreeFlags(u8);

impl AvlTreeFlags {
    /// Create tree-flags
    pub fn new(insert_allowed: bool, update_allowed: bool, remove_allowed: bool) -> Self {
        let read_only = 0;
        let i = if insert_allowed {
            read_only | 0x01
        } else {
            read_only
        };
        let u = if update_allowed { i | 0x02 } else { i };
        AvlTreeFlags(if remove_allowed { u | 0x04 } else { u })
    }

    /// Get byte-representation of the tree-flags
    pub fn serialize(&self) -> u8 {
        self.0
    }

    /// Parse tree-flags from byte
    pub fn parse(serialized_flags: u8) -> Self {
        let insert_allowed = serialized_flags & 0x01 != 0;
        let update_allowed = serialized_flags & 0x02 != 0;
        let remove_allowed = serialized_flags & 0x04 != 0;
        Self::new(insert_allowed, update_allowed, remove_allowed)
    }

    /// Returns true if inserting is allowed
    pub fn insert_allowed(&self) -> bool {
        self.0 & 0x01 != 0
    }

    /// Returns true if updating is allowed
    pub fn update_allowed(&self) -> bool {
        self.0 & 0x02 != 0
    }

    /// Returns true if removal is allowed
    pub fn remove_allowed(&self) -> bool {
        self.0 & 0x04 != 0
    }
}

/// Type of data which efficiently authenticates potentially huge dataset having key-value
/// dictionary interface. Only root hash of dynamic AVL+ tree, tree height, key length, optional
/// value length, and access flags are stored in an instance of the datatype.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AvlTreeData {
    /// Authenticated tree digest: root hash along with tree height
    pub digest: ADDigest,
    /// Allowed modifications
    pub tree_flags: AvlTreeFlags,
    /// All the elements under the tree have the same length
    pub key_length: u32,
    /// If non-empty, all the values under the tree are of the same length
    pub value_length_opt: Option<Box<u32>>,
}

impl SigmaSerializable for AvlTreeData {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.digest.scorex_serialize(w)?;
        w.put_u8(self.tree_flags.0)?;
        w.put_u32(self.key_length)?;
        self.value_length_opt.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let digest = ADDigest::scorex_parse(r)?;
        let tree_flags = AvlTreeFlags::parse(r.get_u8()?);
        let key_length = r.get_u32()?;
        let value_length_opt = <Option<Box<u32>> as SigmaSerializable>::sigma_parse(r)?;
        Ok(AvlTreeData {
            digest,
            tree_flags,
            key_length,
            value_length_opt,
        })
    }
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod arbitrary {

    use super::*;
    use proptest::prelude::*;

    type OptBox = Option<Box<u32>>;
    impl Arbitrary for AvlTreeData {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (
                any::<ADDigest>(),
                any::<AvlTreeFlags>(),
                any::<u32>(),
                any::<OptBox>(),
            )
                .prop_map(
                    |(digest, tree_flags, key_length, value_length_opt)| AvlTreeData {
                        digest,
                        tree_flags,
                        key_length,
                        value_length_opt,
                    },
                )
                .boxed()
        }
    }
    impl Arbitrary for AvlTreeFlags {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            (any::<bool>(), any::<bool>(), any::<bool>())
                .prop_map(|(insert_allowed, update_allowed, remove_allowed)| {
                    AvlTreeFlags::new(insert_allowed, update_allowed, remove_allowed)
                })
                .boxed()
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<AvlTreeData>()) {
            let expr = Expr::Const(v.into());
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}

use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SigmaParsingError,
    SigmaSerializable, SigmaSerializationError, SigmaSerializeResult,
};

use super::constant::TryExtractFromError;

#[derive(PartialEq, Eq, Debug, Clone)]
struct AvlTreeFlags(u8);

impl AvlTreeFlags {
    fn new(insert_allowed: bool, update_allowed: bool, remove_allowed: bool) -> Self {
        let read_only = 0;
        let i = if insert_allowed {
            read_only | 0x01
        } else {
            read_only
        };
        let u = if update_allowed { i | 0x02 } else { i };
        AvlTreeFlags(if remove_allowed { u | 0x04 } else { u })
    }
    fn apply(serialized_flags: u8) -> Self {
        let insert_allowed = serialized_flags & 0x01 != 0;
        let update_allowed = serialized_flags & 0x02 != 0;
        let remove_allowed = serialized_flags & 0x04 != 0;
        Self::new(insert_allowed, update_allowed, remove_allowed)
    }
}

/// Type of data which efficiently authenticates potentially huge dataset having key-value
/// dictionary interface. Only root hash of dynamic AVL+ tree, tree height, key length, optional
/// value length, and access flags are stored in an instance of the datatype.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct AvlTreeData {
    /// Authenticated tree digest: root hash along with tree height
    digest: ADDigest,
    /// Allowed modifications
    tree_flags: AvlTreeFlags,
    /// All the elements under the tree have the same length
    key_length: u32,
    /// If non-empty, all the values under the tree are of the same length
    value_length_opt: Option<u32>,
}

impl SigmaSerializable for AvlTreeData {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.digest.sigma_serialize(w)?;
        w.put_u8(self.tree_flags.0)?;
        w.put_u32(self.key_length)?;
        if let Some(l) = self.value_length_opt {
            w.put_u8(1)?;
            w.put_u32(l)?;
        } else {
            w.put_u8(0)?;
        }
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let digest = ADDigest::sigma_parse(r)?;
        let tree_flags = AvlTreeFlags::apply(r.get_u8()?);
        let key_length = r.get_u32()?;
        let is_some = {
            let s = r.get_u8()?;
            if s == 1 {
                true
            } else if s == 0 {
                false
            } else {
                return Err(SigmaParsingError::SerializationError(
                    SigmaSerializationError::UnexpectedValue(TryExtractFromError(format!(
                        "AvlTreeData: Expected valueLength tag to be 0 or 1, got {}",
                        s
                    ))),
                ));
            }
        };
        let value_length_opt = if is_some {
            let len = r.get_u32()?;
            Some(len)
        } else {
            None
        };
        Ok(AvlTreeData {
            digest,
            tree_flags,
            key_length,
            value_length_opt,
        })
    }
}

/// Digest
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Digest<const N: usize>(pub Box<[u8; N]>);

/// 32 byte array used as ID of some value: block, transaction, etc.
/// Usually this is as blake2b hash of serialized form
pub type Digest32 = Digest<32>;

/// AVL tree digest: root hash along with tree height (33 bytes)
pub type ADDigest = Digest<33>;

impl<const N: usize> SigmaSerializable for Digest<N> {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.write_all(self.0.as_ref())?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let mut bytes = [0; N];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes.into()))
    }
}

//! Sigma byte stream writer
use crate::ergo_tree::ErgoTreeVersion;

use super::constant_store::ConstantStore;
use super::val_def_type_store::ValDefTypeStore;
use super::SigmaParsingError;
use core3::io::Cursor;
use core3::io::Read;
use core3::io::Seek;
use sigma_ser::vlq_encode::PositionLimit;
use sigma_ser::vlq_encode::ReadSigmaVlqExt;

/// Maximum nesting level of value deserialization on one reader
/// (sigmastate `SigmaConstants.MaxTreeDepth`, the default `maxTreeDepth` of every reader)
pub const MAX_TREE_DEPTH: usize = 110;

/// Implementation of SigmaByteRead
pub struct SigmaByteReader<R> {
    inner: R,
    constant_store: ConstantStore,
    substitute_placeholders: bool,
    val_def_type_store: ValDefTypeStore,
    was_deserialize: bool,
    version: ErgoTreeVersion,
    position_limit: u64,
    level: usize,
}

impl<R: Read> SigmaByteReader<R> {
    /// Create new reader from PeekableReader
    pub fn new(pr: R, constant_store: ConstantStore) -> SigmaByteReader<R> {
        SigmaByteReader {
            inner: pr,
            constant_store,
            substitute_placeholders: false,
            val_def_type_store: ValDefTypeStore::new(),
            was_deserialize: false,
            version: ErgoTreeVersion::V0,
            position_limit: u64::MAX,
            level: 0,
        }
    }

    /// Make a new reader with underlying PeekableReader and constant_store to resolve constant
    /// placeholders
    pub fn new_with_substitute_placeholders(
        pr: R,
        constant_store: ConstantStore,
    ) -> SigmaByteReader<R> {
        SigmaByteReader {
            inner: pr,
            constant_store,
            substitute_placeholders: true,
            val_def_type_store: ValDefTypeStore::new(),
            was_deserialize: false,
            version: ErgoTreeVersion::MAX_SCRIPT_VERSION,
            position_limit: u64::MAX,
            level: 0,
        }
    }
}

/// Create SigmaByteReader from a byte array (with empty constant store)
pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> SigmaByteReader<Cursor<T>> {
    SigmaByteReader::new(Cursor::new(bytes), ConstantStore::empty())
}

/// Sigma byte reader trait with a constant store to resolve segregated constants
pub trait SigmaByteRead: ReadSigmaVlqExt {
    /// Constant store with constants to resolve constant placeholder types
    fn constant_store(&mut self) -> &mut ConstantStore;

    /// Option to substitute ConstantPlaceholder with Constant from the store
    fn substitute_placeholders(&self) -> bool;

    /// Set new constant store
    fn set_constant_store(&mut self, constant_store: ConstantStore);

    /// ValDef types store (resolves tpe on ValUse parsing)
    fn val_def_type_store(&mut self) -> &mut ValDefTypeStore;

    /// Returns if value that was deserialized has deserialize nodes, such as DeserializeContext and DeserializeRegister
    fn was_deserialize(&self) -> bool;

    /// Set that deserialization node was read
    fn set_deserialize(&mut self, has_deserialize: bool);

    /// Get position of reader in buffer. This is functionally equivalent to [`std::io::Seek::stream_position`] but redefined here so it can be used in no_std contexts
    fn position(&mut self) -> core3::io::Result<u64> {
        #[cfg(feature = "std")]
        {
            <Self as Seek>::stream_position(self)
        }
        #[cfg(not(feature = "std"))]
        {
            self.seek(core3::io::SeekFrom::Current(0))
        }
    }

    /// Call `f` with reader's ErgoTree version set to `version` inside f's scope
    fn with_tree_version<T>(
        &mut self,
        version: ErgoTreeVersion,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T;

    /// Maximum ErgoTree version that deserializer can handle.
    fn tree_version(&self) -> ErgoTreeVersion;

    /// Current nesting level of value deserialization (sigmastate `CoreByteReader.level`)
    fn level(&self) -> usize;

    /// Set the nesting level. A level above [`MAX_TREE_DEPTH`] fails the parse
    /// (sigmastate v6.0.6 `CoreByteReader.level_=`, `CoreByteReader.scala:127-131`).
    fn set_level(&mut self, level: usize) -> Result<(), SigmaParsingError>;

    /// Call `f` with the parse state a new reader over the same stream starts with:
    /// level 0, empty constant and `ValDef` type stores, the deserialize flag clear
    /// (sigmastate `SigmaByteReader`/`CoreByteReader` construction). The previous state
    /// comes back after `f`, whatever `f` returned. Position, position limit, tree
    /// version and placeholder substitution are left as they are.
    fn with_fresh_parse_state<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T;
}

impl<R: Read> Read for SigmaByteReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> core3::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: Seek> Seek for SigmaByteReader<R> {
    fn seek(&mut self, pos: core3::io::SeekFrom) -> core3::io::Result<u64> {
        self.inner.seek(pos)
    }

    #[cfg(feature = "std")]
    fn rewind(&mut self) -> core3::io::Result<()> {
        self.inner.rewind()
    }

    #[cfg(feature = "std")]
    fn stream_position(&mut self) -> core3::io::Result<u64> {
        self.inner.stream_position()
    }
}

/// Real position-limit storage: this is the decorating reader the limit lives on
/// (the reference impl's `CoreByteReader.positionLimit`), while the wrapped inner
/// reader stays unchecked.
impl<R> PositionLimit for SigmaByteReader<R> {
    fn position_limit(&self) -> u64 {
        self.position_limit
    }
    fn set_position_limit(&mut self, limit: u64) {
        self.position_limit = limit;
    }
}

impl<R: ReadSigmaVlqExt> SigmaByteRead for SigmaByteReader<R> {
    fn constant_store(&mut self) -> &mut ConstantStore {
        &mut self.constant_store
    }

    fn substitute_placeholders(&self) -> bool {
        self.substitute_placeholders
    }

    fn set_constant_store(&mut self, constant_store: ConstantStore) {
        self.constant_store = constant_store;
    }

    fn val_def_type_store(&mut self) -> &mut ValDefTypeStore {
        &mut self.val_def_type_store
    }

    fn was_deserialize(&self) -> bool {
        self.was_deserialize
    }

    fn set_deserialize(&mut self, has_deserialize: bool) {
        self.was_deserialize = has_deserialize
    }

    fn with_tree_version<T>(
        &mut self,
        version: ErgoTreeVersion,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let tmp = self.version;
        self.version = version;
        let res = f(self);
        self.version = tmp;
        res
    }

    fn tree_version(&self) -> ErgoTreeVersion {
        self.version
    }

    fn level(&self) -> usize {
        self.level
    }

    fn set_level(&mut self, level: usize) -> Result<(), SigmaParsingError> {
        if level > MAX_TREE_DEPTH {
            return Err(SigmaParsingError::DeserializeCallDepthExceeded(level));
        }
        self.level = level;
        Ok(())
    }

    fn with_fresh_parse_state<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let constant_store = core::mem::replace(&mut self.constant_store, ConstantStore::empty());
        let val_def_type_store = core::mem::take(&mut self.val_def_type_store);
        let was_deserialize = core::mem::replace(&mut self.was_deserialize, false);
        let level = core::mem::replace(&mut self.level, 0);
        let res = f(self);
        self.constant_store = constant_store;
        self.val_def_type_store = val_def_type_store;
        self.was_deserialize = was_deserialize;
        self.level = level;
        res
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod level_tests {
    use super::*;

    #[test]
    fn set_level_allows_max_tree_depth_and_rejects_above() {
        // `CoreByteReader.level_=` (`CoreByteReader.scala:127-131`): 110 is allowed,
        // 111 fails and leaves the level where it was.
        let mut r = from_bytes([0u8; 0]);
        assert_eq!(r.level(), 0);
        r.set_level(MAX_TREE_DEPTH).unwrap();
        assert!(matches!(
            r.set_level(MAX_TREE_DEPTH + 1),
            Err(SigmaParsingError::DeserializeCallDepthExceeded(111))
        ));
        assert_eq!(r.level(), MAX_TREE_DEPTH);
    }

    #[test]
    fn with_fresh_parse_state_starts_fresh_and_restores() {
        use crate::mir::val_def::ValId;
        use crate::types::stype::SType;

        let mut r = from_bytes([0u8; 0]);
        r.set_level(7).unwrap();
        r.set_constant_store(ConstantStore::new(vec![1i32.into()]));
        r.val_def_type_store().insert(ValId(1), SType::SInt);
        r.set_deserialize(true);
        r.with_fresh_parse_state(|r| {
            assert_eq!(r.level(), 0);
            assert!(r.constant_store().get(0).is_none());
            assert!(r.val_def_type_store().get(&ValId(1)).is_none());
            assert!(!r.was_deserialize());
            r.set_level(3).unwrap();
            r.set_constant_store(ConstantStore::new(vec![true.into()]));
            r.val_def_type_store().insert(ValId(2), SType::SBoolean);
            r.set_deserialize(true);
        });
        assert_eq!(r.level(), 7);
        assert_eq!(r.constant_store().get(0).unwrap().tpe, SType::SInt);
        assert_eq!(r.val_def_type_store().get(&ValId(1)), Some(&SType::SInt));
        assert!(r.val_def_type_store().get(&ValId(2)).is_none());
        assert!(r.was_deserialize());
    }
}

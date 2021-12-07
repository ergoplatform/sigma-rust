//! Derivation path according to
//! BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
//! and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>

/// Index for hardened derivation
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ChildIndexHardened(u32);

impl ChildIndexHardened {
    /// Create new from a 31-bit value (32th bit should not be set)
    pub fn from_31_bit(i: u32) -> Result<Self, ChildIndexError> {
        if i & (1 << 31) == 0 {
            Ok(ChildIndexHardened(i))
        } else {
            Err(ChildIndexError::NumberTooLarge(i))
        }
    }
}

/// Index for normal(non-hardened) derivation
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ChildIndexNormal(u32);

impl ChildIndexNormal {
    /// Create an index for normal (non-hardened) derivation from 31-bit value(32th bit should not be set)
    pub fn normal(i: u32) -> Result<Self, ChildIndexError> {
        if i & (1 << 31) == 0 {
            Ok(ChildIndexNormal(i))
        } else {
            Err(ChildIndexError::NumberTooLarge(i))
        }
    }
}

/// Child index for derivation
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ChildIndex {
    /// Index for hardened derivation
    Hardened(ChildIndexHardened),
    /// Index for normal(non-hardened) derivation
    Normal(ChildIndexNormal),
}

const PURPOSE: ChildIndex = ChildIndex::Hardened(ChildIndexHardened(44));
const ERG: ChildIndex = ChildIndex::Hardened(ChildIndexHardened(429));
/// According to EIP-3 change is always 0 (external address)
const CHANGE: ChildIndex = ChildIndex::Normal(ChildIndexNormal(0));

/// Child index related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildIndexError {
    /// Number is too large
    NumberTooLarge(u32),
}

impl ChildIndex {
    /// Create an index for normal (non-hardened) derivation from 31-bit value(32th bit should not be set)
    pub fn normal(i: u32) -> Result<Self, ChildIndexError> {
        Ok(ChildIndex::Normal(ChildIndexNormal::normal(i)?))
    }

    /// Create an index for hardened derivation from 31-bit value(32th bit should not be set)
    pub fn hardened(i: u32) -> Result<Self, ChildIndexError> {
        Ok(ChildIndex::Hardened(ChildIndexHardened::from_31_bit(i)?))
    }

    /// Return 32-bit representation with highest bit set for hard derivation and clear for normal
    /// derivation
    pub fn to_bits(&self) -> u32 {
        match self {
            ChildIndex::Hardened(index) => (1 << 31) | index.0,
            ChildIndex::Normal(index) => index.0,
        }
    }
}

/// According to
/// BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
/// and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DerivationPath(Box<[ChildIndex]>);

impl DerivationPath {
    /// Create derivation path for a given account index (hardened) and address indices
    /// `m / 44' / 429' / acc' / 0 / address[0] / address[1] / ...`
    /// or `m / 44' / 429' / acc' / 0` if address indices are empty
    /// change is always zero according to EIP-3
    pub fn new(acc: ChildIndexHardened, address_indices: Vec<ChildIndexNormal>) -> Self {
        let mut res = vec![PURPOSE, ERG, ChildIndex::Hardened(acc), CHANGE];
        res.append(
            address_indices
                .into_iter()
                .map(ChildIndex::Normal)
                .collect::<Vec<ChildIndex>>()
                .as_mut(),
        );
        Self(res.into_boxed_slice())
    }

    /// Create root derivation path
    pub fn master_path() -> Self {
        Self(Box::new([]))
    }

    /// For 0x21 Sign Transaction command of Ergo Ledger App Protocol
    /// P2PK Sign (0x0D) instruction
    /// Sign calculated TX hash with private key for provided BIP44 path.
    /// Data:
    ///
    /// Field
    /// Size (B)
    /// Description
    ///
    /// BIP32 path length
    /// 1
    /// Value: 0x02-0x0A (2-10). Number of path components
    ///
    /// First derivation index
    /// 4
    /// Big-endian. Value: 44’
    ///
    /// Second derivation index
    /// 4
    /// Big-endian. Value: 429’ (Ergo coin id)
    ///
    /// Optional Third index
    /// 4
    /// Big-endian. Any valid bip44 hardened value.
    /// ...
    /// Optional Last index
    /// 4
    /// Big-endian. Any valid bip44 value.
    ///
    pub fn ledger_bytes(&self) -> Vec<u8> {
        let mut res = vec![self.0.len() as u8];
        self.0
            .iter()
            .for_each(|i| res.append(&mut i.to_bits().to_be_bytes().to_vec()));
        res
    }

    /// Extend the path with the given index.
    /// Returns this derivation path with added index.
    pub fn extend(&self, index: ChildIndexNormal) -> DerivationPath {
        let mut res = self.0.to_vec();
        res.push(ChildIndex::Normal(index));
        DerivationPath(res.into_boxed_slice())
    }
}

impl ChildIndexNormal {
    /// Return next index value (incremented)
    pub fn next(&self) -> ChildIndexNormal {
        ChildIndexNormal(self.0 + 1)
    }
}

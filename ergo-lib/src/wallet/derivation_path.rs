//! Derivation path according to
//! BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
//! and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>

/// Index for hardened derivation
pub struct ChildIndexHardened(u32);
/// Index for normal(non-hardened) derivation
pub struct ChildIndexNormal(u32);

/// Child index for derivation
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
const ZERO: ChildIndex = ChildIndex::Normal(ChildIndexNormal(0));

/// Child index related errors
pub enum ChildIndexError {
    /// Nomber is too large
    NumberTooLarge(u32),
}

impl ChildIndex {
    /// Create an index for normal (non-hardened) derivation
    pub fn normal(i: u32) -> Result<Self, ChildIndexError> {
        if i & (1 << 31) == 0 {
            Ok(ChildIndex::Normal(ChildIndexNormal(i)))
        } else {
            Err(ChildIndexError::NumberTooLarge(i))
        }
    }

    /// Create an index for hardened derivation
    pub fn hardened(i: u32) -> Result<Self, ChildIndexError> {
        if i & (1 << 31) == 0 {
            Ok(ChildIndex::Hardened(ChildIndexHardened(i)))
        } else {
            Err(ChildIndexError::NumberTooLarge(i))
        }
    }
}

/// According to
/// BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
/// and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>
pub struct DerivationPath(Box<[ChildIndex]>);

impl DerivationPath {
    /// Create derivation path for a given account index (hardened)
    /// m / 44' / 429' / acc' / 0 / 0
    pub fn from_acc_num(acc: ChildIndexHardened) -> Self {
        Self([PURPOSE, ERG, ChildIndex::Hardened(acc), CHANGE, ZERO].into())
    }

    /// Create derivation path for a given account index (hardened) and address index
    /// m / 44' / 429' / acc' / 0 / address
    pub fn new(acc: ChildIndexHardened, address: ChildIndexNormal) -> Self {
        Self(
            [
                PURPOSE,
                ERG,
                ChildIndex::Hardened(acc),
                CHANGE,
                ChildIndex::Normal(address),
            ]
            .into(),
        )
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
    /// [Optional] Third index
    /// 4
    /// Big-endian. Any valid bip44 hardened value.
    /// ...
    /// [Optional] Last index
    /// 4
    /// Big-endian. Any valid bip44 value.
    ///
    pub fn ledger_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

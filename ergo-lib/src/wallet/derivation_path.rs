//! Derivation path according to
//! BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
//! and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>

use derive_more::From;
use std::{collections::VecDeque, fmt, num::ParseIntError, str::FromStr};
use thiserror::Error;

/// Index for hardened derivation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

    /// Return the next child index (incremented)
    pub fn next(&self) -> Result<Self, ChildIndexError> {
        ChildIndexHardened::from_31_bit(self.0 + 1)
    }
}

/// Index for normal(non-hardened) derivation
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

    /// Return next index value (incremented)
    pub fn next(&self) -> ChildIndexNormal {
        ChildIndexNormal(self.0 + 1)
    }
}

/// Child index for derivation
#[derive(PartialEq, Eq, Clone, Copy, Debug, From)]
pub enum ChildIndex {
    /// Index for hardened derivation
    Hardened(ChildIndexHardened),
    /// Index for normal(non-hardened) derivation
    Normal(ChildIndexNormal),
}

impl fmt::Display for ChildIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildIndex::Hardened(i) => write!(f, "{}'", i.0.to_string()),
            ChildIndex::Normal(i) => write!(f, "{}", i.0.to_string()),
        }
    }
}

impl FromStr for ChildIndex {
    type Err = ChildIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('\'') {
            let idx = s.replace("'", "");
            Ok(ChildIndex::Hardened(ChildIndexHardened::from_31_bit(
                idx.parse()?,
            )?))
        } else {
            Ok(ChildIndex::Normal(ChildIndexNormal::normal(s.parse()?)?))
        }
    }
}

const PURPOSE: ChildIndex = ChildIndex::Hardened(ChildIndexHardened(44));
const ERG: ChildIndex = ChildIndex::Hardened(ChildIndexHardened(429));
/// According to EIP-3 change is always 0 (external address)
const CHANGE: ChildIndex = ChildIndex::Normal(ChildIndexNormal(0));

/// Child index related errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ChildIndexError {
    /// Number is too large
    #[error("number too large: {0}")]
    NumberTooLarge(u32),
    /// Provided derivation path contained invalid integer indices
    #[error("failed to parse index: {0}")]
    BadIndex(#[from] ParseIntError),
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

    /// Returns a new instance of the `ChildIndex` with the index incremented
    pub fn next(&self) -> Result<Self, ChildIndexError> {
        match self {
            ChildIndex::Hardened(i) => Ok(ChildIndex::Hardened(i.next()?)),
            ChildIndex::Normal(i) => Ok(ChildIndex::Normal(i.next())),
        }
    }
}

/// According to
/// BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
/// and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub struct DerivationPath(pub(super) Box<[ChildIndex]>);

/// DerivationPath errors
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DerivationPathError {
    /// Provided derivation path was empty
    /// For example, when parsing a path from a string
    #[error("derivation path is empty")]
    EmptyPath,
    /// Provided derivation path was in the wrong format
    /// For example, parsing from string to DerivationPath might have been missing the leading `m`
    #[error("invalid derivation path format")]
    InvalidFormat(String),
    /// There was an issue with one of the children in the path
    #[error("child error: {0}")]
    ChildIndex(#[from] ChildIndexError),
}

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

    /// Returns the length of the derivation path
    pub fn depth(&self) -> usize {
        self.0.len()
    }

    /// Extend the path with the given index.
    /// Returns this derivation path with added index.
    pub fn extend(&self, index: ChildIndex) -> DerivationPath {
        let mut res = self.0.to_vec();
        res.push(index);
        DerivationPath(res.into_boxed_slice())
    }

    /// Returns a new path with the last element of the deriviation path being increased, e.g. m/1/2 -> m/1/3
    /// Returns an empty path error if the path is empty (master node for example)
    pub fn next(&self) -> Result<DerivationPath, DerivationPathError> {
        #[allow(clippy::unwrap_used)]
        if self.0.len() > 0 {
            let mut new_path: Vec<_> = self.0.iter().cloned().collect();
            let last_idx = new_path.len() - 1;
            // The bounds have been checked, there is at least one element
            new_path[last_idx] = new_path.last().unwrap().next()?;

            Ok(DerivationPath(new_path.into_boxed_slice()))
        } else {
            Err(DerivationPathError::EmptyPath)
        }
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
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m/")?;
        let children = self
            .0
            .iter()
            .map(ChildIndex::to_string)
            .collect::<Vec<_>>()
            .join("/");
        write!(f, "{}", children)?;

        Ok(())
    }
}

impl FromStr for DerivationPath {
    type Err = DerivationPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cleaned_parts = s.split_whitespace().collect::<String>();
        let mut parts = cleaned_parts.split('/').collect::<VecDeque<_>>();
        let master_key_id = parts.pop_front().ok_or(DerivationPathError::EmptyPath)?;
        if master_key_id != "m" && master_key_id != "M" {
            return Err(DerivationPathError::InvalidFormat(format!(
                "Master node must be either 'm' or 'M', got {}",
                master_key_id
            )));
        }
        let path = parts
            .into_iter()
            .map(ChildIndex::from_str)
            .flatten()
            .collect::<Vec<_>>();
        Ok(path.into_boxed_slice().into())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_derivation_path_to_string() {
        let path = DerivationPath::new(ChildIndexHardened(1), vec![ChildIndexNormal(3)]);
        let expected = "m/44'/429'/1'/0/3";

        assert_eq!(expected, path.to_string())
    }

    #[test]
    fn test_derivation_path_to_string_no_addr() {
        let path = DerivationPath::new(ChildIndexHardened(0), vec![]);
        let expected = "m/44'/429'/0'/0";

        assert_eq!(expected, path.to_string())
    }

    #[test]
    fn test_string_to_derivation_path() {
        let path = "m/44'/429'/0'/0/1";
        let expected = DerivationPath::new(ChildIndexHardened(0), vec![ChildIndexNormal(1)]);

        assert_eq!(expected, path.parse::<DerivationPath>().unwrap())
    }

    #[test]
    fn test_derivation_path_next() {
        // m/44'/429'/1'/0/3
        let path = DerivationPath::new(ChildIndexHardened(1), vec![ChildIndexNormal(3)]);
        let new_path = path.next().unwrap();
        let expected = "m/44'/429'/1'/0/4";

        assert_eq!(expected, new_path.to_string());
    }

    // Test derivation_path.next() returns error if empty (doesn't panic)
    #[test]
    fn test_derivation_path_next_returns_err_if_emtpy() {
        let path = DerivationPath(Box::new([]));

        assert_eq!(path.next(), Err(DerivationPathError::EmptyPath))
    }
}

use ergotree_ir::address::Address;

/// Public key bytes size, as defined in "0x10 - Get the extended public key" of Ergo Ledger App Protocol
pub const PUB_KEY_SIZE: usize = 64;
/// Chain code size, as defined in "0x10 - Get the extended public key" of Ergo Ledger App Protocol
pub const CHAIN_CODE_SIZE: usize = 31;

/// Extended public key
/// see BIP-32
/// and EIP-3 https://github.com/ergoplatform/eips/blob/master/eip-0003.md
pub struct XPub {
    pub_key: [u8; PUB_KEY_SIZE],
    chain_code: [u8; CHAIN_CODE_SIZE],
}

impl XPub {
    /// Create new XPub from public key and chain code bytes
    /// returned by "0x10 - Get the extended public key" of Ergo Ledger App Protocol
    pub fn new(pub_key: [u8; PUB_KEY_SIZE], chain_code: [u8; CHAIN_CODE_SIZE]) -> Self {
        XPub {
            pub_key,
            chain_code,
        }
    }

    /// Derive (soft-only) child extended public key
    /// according to EIP-3 https://github.com/ergoplatform/eips/blob/master/eip-0003.md
    /// for the given address index (M/44'/429'/account'/0/address_index).
    /// Since EIP-3 does not use internal addresses, hence `0` for "change" field
    pub fn derive(&self, address_index: u32) -> Self {
        todo!()
    }
}

impl From<XPub> for Address {
    fn from(_: XPub) -> Self {
        todo!()
    }
}

/// BIP-44 path (M/44'/429'/account'/0/address_index)
/// according to EIP-3 https://github.com/ergoplatform/eips/blob/master/eip-0003.md
/// Since EIP-3 does not use internal addresses, hence `0` for "change" field
pub struct Bip44Path {
    account: u32,
    address_index: u32,
}

impl Bip44Path {
    /// Serialized to bytes according to Ergo Ledger App Protocol
    /// path lenght (1 byte, 0x02-0x0A)
    /// + first index (4 bytes, 44')
    /// + second index (4 bytes, 429')
    /// + third index (4 bytes, account index)
    /// + forth index (4 bytes, 0 (change))
    /// + fifth index (4 bytes, address index)
    /// indexes are encoded as big-endian
    /// TODO: what is "valid BIP-44 hardened value" (OR it with 0x80000000?) and why only starting
    /// with third index?
    pub fn to_ledger_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

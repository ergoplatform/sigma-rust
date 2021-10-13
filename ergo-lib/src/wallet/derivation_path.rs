pub struct Index(u32);

impl Index {
    pub const HARD_RANGE_START: u32 = 0x80000000;

    pub fn hardened(i: u32) -> Self {
        Self(i | Self::HARD_RANGE_START)
    }

    pub fn is_hardened(&self) -> bool {
        (self.0 & Self::HARD_RANGE_START) != 0
    }
}

pub struct DerivationPath {
    public_branch: bool,
    decoded_path: Vec<Index>,
}

impl DerivationPath {
    pub fn from_acc_num(acc_num: u32) -> Self {
        Self {
            decoded_path: vec![44, 429, acc_num, 0, 0],
        }
    }

    /// Change bool
    pub fn new(acc_num: u32, index: u32) -> Self {
        todo!()
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

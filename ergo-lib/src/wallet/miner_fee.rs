//! Miner fee included in transaction

use ergotree_ir::chain::address::Address;
use ergotree_ir::chain::address::AddressEncoder;
use ergotree_ir::chain::address::NetworkPrefix;
use lazy_static::lazy_static;

/// Base16 encoded serialized ErgoTree of the miners fee (delay 720)
pub const MINERS_FEE_BASE16_BYTES: &str = "1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304";

lazy_static! {
    /// Miner fee P2S address on mainnet
    pub static ref MINERS_FEE_ADDRESS: Address =
        #[allow(clippy::unwrap_used)]
        Address::P2S(base16::decode(MINERS_FEE_BASE16_BYTES).unwrap());

    /// Miner fee Base58 encoded P2S address on mainnet
    pub static ref MINERS_FEE_MAINNET_ADDRESS_STR: String =
        AddressEncoder::new(NetworkPrefix::Mainnet).address_to_str(&MINERS_FEE_ADDRESS);

    /// Miner fee Base58 encoded P2S address on testnet
    pub static ref MINERS_FEE_TESTNET_ADDRESS_STR: String =
        AddressEncoder::new(NetworkPrefix::Testnet).address_to_str(&MINERS_FEE_ADDRESS);
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {

    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::serialization::SigmaSerializable;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn fee_mainnet_address() {
        assert_eq!(
            MINERS_FEE_MAINNET_ADDRESS_STR.as_str(),
            "2iHkR7CWvD1R4j1yZg5bkeDRQavjAaVPeTDFGGLZduHyfWMuYpmhHocX8GJoaieTx78FntzJbCBVL6rf96ocJoZdmWBL2fci7NqWgAirppPQmZ7fN9V6z13Ay6brPriBKYqLp1bT2Fk4FkFLCfdPpe"
        );
    }

    #[test]
    fn fee_testnet_address() {
        assert_eq!(
            MINERS_FEE_TESTNET_ADDRESS_STR.as_str(),
            "Bf1X9JgQTUtgntaer91B24n6kP8L2kqEiQqNf1z97BKo9UbnW3WRP9VXu8BXd1LsYCiYbHJEdWKxkF5YNx5n7m31wsDjbEuB3B13ZMDVBWkepGmWfGa71otpFViHDCuvbw1uNicAQnfuWfnj8fbCa4"
        );
    }

    #[test]
    fn parses_fee_address() {
        ErgoTree::sigma_parse_bytes(base16::decode(MINERS_FEE_BASE16_BYTES).unwrap().as_slice())
            .unwrap();
    }
}

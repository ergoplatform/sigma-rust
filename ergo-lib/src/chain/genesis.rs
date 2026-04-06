//! Genesis block construction for the Ergo blockchain.
//!
//! Constructs the three genesis boxes (emission, no-premine, foundation)
//! from chain parameters, matching the JVM reference implementation.

use alloc::vec;
use alloc::vec::Vec;

use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::ergo_box::box_value::BoxValueError;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::ergo_box::NonMandatoryRegisterId;
use ergotree_ir::chain::ergo_box::NonMandatoryRegisters;
use ergotree_ir::chain::ergo_box::NonMandatoryRegistersError;
use ergotree_ir::chain::tx_id::TxId;
use ergotree_ir::mir::atleast::Atleast;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::expr::InvalidArgumentError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
use ergotree_ir::types::stype::SType;

use super::emission::EmissionRules;
use super::emission::MonetarySettings;
use super::emission::COINS_IN_ONE_ERGO;
use super::ergo_tree_predef;
use super::ergo_tree_predef::ErgoTreePredefError;

/// Errors that can occur during genesis box construction.
#[derive(Debug, thiserror::Error)]
pub enum GenesisError {
    /// Error constructing a predefined ErgoTree.
    #[error("ergo tree predef error: {0}")]
    ErgoTreePredef(#[from] ErgoTreePredefError),
    /// Error creating a box value.
    #[error("box value error: {0}")]
    BoxValue(#[from] BoxValueError),
    /// Error serializing a box (for box ID computation).
    #[error("serialization error: {0}")]
    Serialization(#[from] SigmaSerializationError),
    /// Error creating non-mandatory registers.
    #[error("register error: {0}")]
    Register(#[from] NonMandatoryRegistersError),
    /// Error constructing an IR expression.
    #[error("invalid argument: {0}")]
    InvalidArgument(#[from] InvalidArgumentError),
}

/// Register IDs for no-premine proof strings (R4 through R8).
const PROOF_REGISTER_IDS: [NonMandatoryRegisterId; 5] = [
    NonMandatoryRegisterId::R4,
    NonMandatoryRegisterId::R5,
    NonMandatoryRegisterId::R6,
    NonMandatoryRegisterId::R7,
    NonMandatoryRegisterId::R8,
];

/// Build the founders box R4 constant from a k-of-n threshold over public keys.
///
/// The JVM reference serializes `Atleast(k, Coll(pk1, pk2, ...))` at the
/// **Expr level** using `ValueSerializer.serialize`, then wraps the bytes
/// in a `ByteArrayConstant`. This function reproduces that encoding.
fn build_founders_r4(founders_pks: &[ProveDlog], threshold: u8) -> Result<Constant, GenesisError> {
    let pk_exprs: Vec<Expr> = founders_pks
        .iter()
        .map(|pk| Expr::Const(Constant::from(SigmaProp::from(pk.clone()))))
        .collect();
    let bound = Expr::Const(Constant::from(i32::from(threshold)));
    let input = Collection::new(SType::SSigmaProp, pk_exprs)?;
    let atleast = Atleast::new(bound, Expr::Collection(input))?;
    let atleast_bytes = Expr::Atleast(atleast).sigma_serialize_bytes()?;
    Ok(Constant::from(atleast_bytes))
}

/// Construct the three genesis boxes for an Ergo network.
///
/// Returns `(emission_box, no_premine_box, founders_box)`.
///
/// # Parameters
///
/// - `settings`: monetary parameters for the chain
/// - `founders_pks`: public keys for the foundation multisig
/// - `founders_threshold`: minimum number of founder signatures required
/// - `no_premine_proofs`: proof-of-no-premine strings stored in the
///   no-premine box registers R4–R8 (e.g. Bitcoin block hash, news
///   headlines). At most 5 strings; extras are silently ignored.
///
/// All three genesis boxes use `TxId::zero()`, `creation_height = 0`,
/// and `index = 0` (matching the JVM reference, which treats each as the
/// first output of a virtual genesis transaction).
pub fn genesis_boxes(
    settings: &MonetarySettings,
    founders_pks: &[ProveDlog],
    founders_threshold: u8,
    no_premine_proofs: &[&str],
) -> Result<(ErgoBox, ErgoBox, ErgoBox), GenesisError> {
    let rules = EmissionRules::new(settings.clone());
    let tx_id = TxId::zero();

    // --- Emission box (index 0) ---
    let emission_box = ErgoBox::new(
        BoxValue::try_from(rules.miners_coins_total() as u64)?,
        ergo_tree_predef::emission_box_prop(settings)?,
        None,
        NonMandatoryRegisters::empty(),
        0,
        tx_id,
        0,
    )?;

    // --- No-premine box (index 0) ---
    let proof_regs: Vec<(NonMandatoryRegisterId, Constant)> = no_premine_proofs
        .iter()
        .zip(PROOF_REGISTER_IDS.iter())
        .map(|(s, reg)| (*reg, Constant::from(s.as_bytes().to_vec())))
        .collect();
    let no_premine_registers = if proof_regs.is_empty() {
        NonMandatoryRegisters::empty()
    } else {
        NonMandatoryRegisters::new(proof_regs)?
    };

    let no_premine_box = ErgoBox::new(
        BoxValue::try_from(COINS_IN_ONE_ERGO as u64)?,
        ergo_tree_predef::false_prop()?,
        None,
        no_premine_registers,
        0,
        tx_id,
        0,
    )?;

    // --- Founders box (index 0) ---
    let founders_r4 = build_founders_r4(founders_pks, founders_threshold)?;
    let founders_registers =
        NonMandatoryRegisters::new(vec![(NonMandatoryRegisterId::R4, founders_r4)])?;
    let founders_value = (rules.founders_coins_total() - COINS_IN_ONE_ERGO) as u64;

    let founders_box = ErgoBox::new(
        BoxValue::try_from(founders_value)?,
        ergo_tree_predef::foundation_script(settings)?,
        None,
        founders_registers,
        0,
        tx_id,
        0,
    )?;

    Ok((emission_box, no_premine_box, founders_box))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use ergo_chain_types::EcPoint;

    /// Testnet no-premine proof strings.
    const TESTNET_PROOFS: &[&str] = &[
        "'Chaos reigns': what the papers say about the no-deal Brexit vote",
        "习近平的两会时间|这里有份习近平两会日历，请查收！",
        "ТАСС сообщил об обнаружении нескольких майнинговых ферм на столичных рынках",
        "000000000000000000139a3e61bd5721827b51a5309a8bfeca0b8c4b5c060931",
        "0xef1d584d77e74e3c509de625dc17893b22b73d040b5d5302bbf832065f928d03",
    ];

    /// Testnet founder public keys (hex-encoded compressed EC points).
    const TESTNET_FOUNDER_PKS: &[&str] = &[
        "039bb5fe52359a64c99a60fd944fc5e388cbdc4d37ff091cc841c3ee79060b8647",
        "031fb52cf6e805f80d97cde289f4f757d49accf0c83fb864b27d2cf982c37f9a8b",
        "0352ac2a471339b0d23b3d2c5ce0db0e81c969f77891b9edf0bda7fd39a78184e7",
    ];

    fn parse_founder_pks() -> Vec<ProveDlog> {
        TESTNET_FOUNDER_PKS
            .iter()
            .map(|hex| {
                let bytes = base16::decode(hex.as_bytes()).unwrap();
                let point = EcPoint::sigma_parse_bytes(&bytes).unwrap();
                ProveDlog::new(point)
            })
            .collect()
    }

    fn build_testnet_genesis() -> (ErgoBox, ErgoBox, ErgoBox) {
        let settings = MonetarySettings::default();
        let pks = parse_founder_pks();
        genesis_boxes(&settings, &pks, 2, TESTNET_PROOFS).unwrap()
    }

    #[test]
    fn test_genesis_emission_box_value() {
        let (emission, _, _) = build_testnet_genesis();
        let rules = EmissionRules::new(MonetarySettings::default());
        assert_eq!(emission.value.as_i64(), rules.miners_coins_total());
    }

    #[test]
    fn test_genesis_no_premine_box_value() {
        let (_, no_premine, _) = build_testnet_genesis();
        assert_eq!(no_premine.value.as_i64(), COINS_IN_ONE_ERGO);
    }

    #[test]
    fn test_genesis_founders_box_value() {
        let (_, _, founders) = build_testnet_genesis();
        let rules = EmissionRules::new(MonetarySettings::default());
        let expected = rules.founders_coins_total() - COINS_IN_ONE_ERGO;
        assert_eq!(founders.value.as_i64(), expected);
    }

    #[test]
    fn test_genesis_total_value_equals_coins_total() {
        let (emission, no_premine, founders) = build_testnet_genesis();
        let rules = EmissionRules::new(MonetarySettings::default());
        let total = emission.value.as_i64() + no_premine.value.as_i64() + founders.value.as_i64();
        assert_eq!(total, rules.coins_total());
    }

    #[test]
    fn test_genesis_all_indices_zero() {
        let (emission, no_premine, founders) = build_testnet_genesis();
        assert_eq!(emission.index, 0);
        assert_eq!(no_premine.index, 0);
        assert_eq!(founders.index, 0);
    }

    #[test]
    fn test_genesis_box_creation_height() {
        let (emission, no_premine, founders) = build_testnet_genesis();
        assert_eq!(emission.creation_height, 0);
        assert_eq!(no_premine.creation_height, 0);
        assert_eq!(founders.creation_height, 0);
    }

    #[test]
    fn test_testnet_emission_box_id() {
        let (emission, _, _) = build_testnet_genesis();
        let id: String = emission.box_id().into();
        assert_eq!(
            id,
            "b69575e11c5c43400bfead5976ee0d6245a1168396b2e2a4f384691f275d501c"
        );
    }

    #[test]
    fn test_testnet_no_premine_box_id() {
        let (_, no_premine, _) = build_testnet_genesis();
        let id: String = no_premine.box_id().into();
        assert_eq!(
            id,
            "3bfaf76c824df668822dfce71abaf688d0281f91c3ac2a271f92fa28c3efaac7"
        );
    }

    #[test]
    fn test_testnet_founders_box_id() {
        let (_, _, founders) = build_testnet_genesis();
        let id: String = founders.box_id().into();
        assert_eq!(
            id,
            "5527430474b673e4aafb08e0079c639de23e6a17e87edd00f78662b43c88aeda"
        );
    }
}

//! Predefined ErgoTree scripts for Ergo consensus.
//!
//! Ports `ErgoTreePredef` from the JVM's `sigmastate-interpreter` to Rust.
//! Each function constructs an [`ErgoTree`] by building the IR expression
//! tree directly (no ErgoScript compilation), matching the JVM reference
//! byte-for-byte.

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use ergo_chain_types::ec_point::generator;
use ergotree_ir::chain::ergo_box::NonMandatoryRegisterId;
use ergotree_ir::chain::ergo_box::RegisterId;
use ergotree_ir::ergo_tree::{ErgoTree, ErgoTreeError, ErgoTreeHeader};
use ergotree_ir::mir::and::And;
use ergotree_ir::mir::bin_op::{ArithOp, BinOp, BinOpKind, RelationOp};
use ergotree_ir::mir::bool_to_sigma::BoolToSigmaProp;
use ergotree_ir::mir::coll_by_index::ByIndex;
use ergotree_ir::mir::coll_size::SizeOf;
use ergotree_ir::mir::collection::Collection;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::mir::create_provedlog::CreateProveDlog;
use ergotree_ir::mir::decode_point::DecodePoint;
use ergotree_ir::mir::deserialize_register::DeserializeRegister;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::mir::expr::InvalidArgumentError;
use ergotree_ir::mir::extract_amount::ExtractAmount;
use ergotree_ir::mir::extract_creation_info::ExtractCreationInfo;
use ergotree_ir::mir::extract_script_bytes::ExtractScriptBytes;
use ergotree_ir::mir::global_vars::GlobalVars;
use ergotree_ir::mir::if_op::If;
use ergotree_ir::mir::or::Or;
use ergotree_ir::mir::select_field::SelectField;
use ergotree_ir::mir::sigma_and::SigmaAnd;
use ergotree_ir::mir::subst_const::SubstConstants;
use ergotree_ir::mir::unary_op::OneArgOpTryBuild;
use ergotree_ir::mir::upcast::Upcast;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializationError;
use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
use ergotree_ir::sigma_protocol::sigma_boolean::SigmaProp;
use ergotree_ir::types::stype::SType;

use super::emission::MonetarySettings;

/// Errors that can occur when constructing predefined ErgoTrees.
#[derive(Debug, thiserror::Error)]
pub enum ErgoTreePredefError {
    /// Invalid argument passed to an IR node constructor.
    #[error("invalid argument: {0}")]
    InvalidArgument(#[from] InvalidArgumentError),
    /// Error constructing an ErgoTree from an expression.
    #[error("ergo tree error: {0}")]
    ErgoTree(#[from] ErgoTreeError),
    /// Error serializing an ErgoTree (used during template construction).
    #[error("serialization error: {0}")]
    Serialization(#[from] SigmaSerializationError),
}

// ---------------------------------------------------------------------------
// Private IR helpers — keep the tree-construction code readable
// ---------------------------------------------------------------------------

fn int_const(v: i32) -> Expr {
    Expr::Const(Constant::from(v))
}

fn long_const(v: i64) -> Expr {
    Expr::Const(Constant::from(v))
}

fn height() -> Expr {
    Expr::GlobalVars(GlobalVars::Height)
}

fn self_box() -> Expr {
    Expr::GlobalVars(GlobalVars::SelfBox)
}

fn outputs() -> Expr {
    Expr::GlobalVars(GlobalVars::Outputs)
}

fn miner_pubkey() -> Expr {
    Expr::GlobalVars(GlobalVars::MinerPubKey)
}

fn plus(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Arith(ArithOp::Plus),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn minus(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Arith(ArithOp::Minus),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn multiply(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Arith(ArithOp::Multiply),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn divide(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Arith(ArithOp::Divide),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn eq(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Relation(RelationOp::Eq),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn lt(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Relation(RelationOp::Lt),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn gt(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Relation(RelationOp::Gt),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn ge(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Relation(RelationOp::Ge),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

fn le(left: Expr, right: Expr) -> Expr {
    BinOp {
        kind: BinOpKind::Relation(RelationOp::Le),
        left: Box::new(left),
        right: Box::new(right),
    }
    .into()
}

/// AND over a collection of boolean expressions.
fn bool_and(items: Vec<Expr>) -> Result<Expr, InvalidArgumentError> {
    let coll = Collection::new(SType::SBoolean, items)?;
    Ok(And {
        input: Box::new(Expr::Collection(coll)),
    }
    .into())
}

/// OR over a collection of boolean expressions.
fn bool_or(items: Vec<Expr>) -> Result<Expr, InvalidArgumentError> {
    let coll = Collection::new(SType::SBoolean, items)?;
    Ok(Or {
        input: Box::new(Expr::Collection(coll)),
    }
    .into())
}

/// Convert a boolean expression to a SigmaProp.
fn to_sigma_prop(bool_expr: Expr) -> Expr {
    Expr::BoolToSigmaProp(BoolToSigmaProp {
        input: Box::new(bool_expr),
    })
}

/// `SelectField(ExtractCreationInfo(box_expr), 1)` — creation height of a box.
fn box_creation_height(box_expr: Expr) -> Result<Expr, ErgoTreePredefError> {
    let creation_info = Expr::ExtractCreationInfo(ExtractCreationInfo {
        input: Box::new(box_expr),
    });
    let field_index = 1u8
        .try_into()
        .map_err(|_| InvalidArgumentError("tuple field index 1 out of bounds".into()))?;
    let field = SelectField::new(creation_info, field_index)?;
    Ok(field.into())
}

fn extract_amount(box_expr: Expr) -> Expr {
    Expr::ExtractAmount(ExtractAmount {
        input: Box::new(box_expr),
    })
}

fn extract_script_bytes(box_expr: Expr) -> Expr {
    Expr::ExtractScriptBytes(ExtractScriptBytes {
        input: Box::new(box_expr),
    })
}

fn size_of(coll_expr: Expr) -> Expr {
    Expr::SizeOf(SizeOf {
        input: Box::new(coll_expr),
    })
}

fn by_index(coll: Expr, index: Expr) -> Result<Expr, InvalidArgumentError> {
    Ok(ByIndex::new(coll, index, None)?.into())
}

fn upcast_to_long(expr: Expr) -> Result<Expr, InvalidArgumentError> {
    Ok(Expr::Upcast(Upcast::new(expr, SType::SLong)?))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// ErgoTree that always evaluates to `false` (spending is impossible).
///
/// Equivalent to JVM `ErgoTreePredef.FalseProp`.
pub fn false_prop() -> Result<ErgoTree, ErgoTreePredefError> {
    let expr = to_sigma_prop(Expr::Const(Constant::from(false)));
    Ok(ErgoTree::new(ErgoTreeHeader::v0(true), &expr)?)
}

/// ErgoTree that always evaluates to `true` (anyone can spend).
///
/// Equivalent to JVM `ErgoTreePredef.TrueProp`.
pub fn true_prop() -> Result<ErgoTree, ErgoTreePredefError> {
    let expr = to_sigma_prop(Expr::Const(Constant::from(true)));
    Ok(ErgoTree::new(ErgoTreeHeader::v0(true), &expr)?)
}

/// Miner reward output script: spendable after `delta` blocks by the miner.
///
/// ```text
/// SigmaAnd(
///   GE(HEIGHT, boxCreationHeight(SELF) + delta).toSigmaProp,
///   SigmaPropConstant(minerPk)
/// )
/// ```
///
/// Equivalent to JVM `ErgoTreePredef.rewardOutputScript`.
pub fn reward_output_script(
    delta: i32,
    miner_pk: ProveDlog,
) -> Result<ErgoTree, ErgoTreePredefError> {
    let height_check = ge(
        height(),
        plus(box_creation_height(self_box())?, int_const(delta)),
    );
    let miner_prop: Expr = Expr::Const(SigmaProp::from(miner_pk).into());
    let expr = Expr::SigmaAnd(SigmaAnd::new(vec![
        to_sigma_prop(height_check),
        miner_prop,
    ])?);
    Ok(ErgoTree::new(ErgoTreeHeader::v0(true), &expr)?)
}

/// Build the `SubstConstants` expression that produces the expected miner
/// output script bytes at evaluation time.
///
/// Creates a generic reward script template (using the generator point),
/// serializes it, then uses `SubstConstants` to replace the miner PK
/// constant (at position 1) with the actual miner's public key decoded
/// from `miner_pk_bytes_val`.
///
/// Equivalent to JVM `ErgoTreePredef.expectedMinerOutScriptBytesVal`.
pub fn expected_miner_out_script_bytes_val(
    delta: i32,
    miner_pk_bytes_val: Expr,
) -> Result<Expr, ErgoTreePredefError> {
    let generic_pk = ProveDlog::new(generator());
    let generic_miner_prop = reward_output_script(delta, generic_pk)?;
    let generic_miner_prop_bytes = generic_miner_prop.sigma_serialize_bytes()?;

    let positions: Expr = Expr::Const(Constant::from(vec![1i32]));

    let decoded_point = Expr::DecodePoint(DecodePoint::try_build(miner_pk_bytes_val)?);
    let miner_pubkey_sigma_prop = Expr::CreateProveDlog(CreateProveDlog::try_build(decoded_point)?);
    let new_vals = Collection::new(SType::SSigmaProp, vec![miner_pubkey_sigma_prop])?;

    let subst = SubstConstants::new(
        Expr::Const(Constant::from(generic_miner_prop_bytes)),
        positions,
        Expr::Collection(new_vals),
    )?;
    Ok(subst.into())
}

/// Fee proposition: a box spendable only in the same block it was created,
/// with a single output matching the expected miner reward script.
///
/// ```text
/// AND(
///   EQ(HEIGHT, boxCreationHeight(Outputs(0))),
///   EQ(ExtractScriptBytes(Outputs(0)), expectedMinerOutScriptBytesVal(delta, MinerPubkey)),
///   EQ(SizeOf(Outputs), 1)
/// ).toSigmaProp
/// ```
///
/// Equivalent to JVM `ErgoTreePredef.feeProposition`.
pub fn fee_proposition(delta: i32) -> Result<ErgoTree, ErgoTreePredefError> {
    let out = by_index(outputs(), int_const(0))?;

    let height_ok = eq(height(), box_creation_height(out.clone())?);
    let script_ok = eq(
        extract_script_bytes(out),
        expected_miner_out_script_bytes_val(delta, miner_pubkey())?,
    );
    let outputs_ok = eq(size_of(outputs()), int_const(1));

    let prop = to_sigma_prop(bool_and(vec![height_ok, script_ok, outputs_ok])?);
    Ok(ErgoTree::new(ErgoTreeHeader::v0(true), &prop)?)
}

/// Emission box proposition: controls the release of miner rewards over time.
///
/// Equivalent to JVM `ErgoTreePredef.emissionBoxProp`.
pub fn emission_box_prop(s: &MonetarySettings) -> Result<ErgoTree, ErgoTreePredefError> {
    let reward_out = by_index(outputs(), int_const(0))?;
    let miner_out = by_index(outputs(), int_const(1))?;

    let miners_reward = s.fixed_rate - s.founders_initial_reward;
    let miners_fixed_rate_period = i64::from(s.fixed_rate_period) + 2 * i64::from(s.epoch_length);

    // epoch = 1 + (HEIGHT - fixedRatePeriod) / epochLength
    let epoch = plus(
        int_const(1),
        divide(
            minus(height(), int_const(s.fixed_rate_period)),
            int_const(s.epoch_length),
        ),
    );

    // coinsToIssue = If(HEIGHT < minersFixedRatePeriod, minersReward,
    //                    fixedRate - oneEpochReduction * epoch.toLong)
    let coins_to_issue = Expr::If(If {
        condition: Box::new(lt(height(), int_const(miners_fixed_rate_period as i32))),
        true_branch: Box::new(long_const(miners_reward)),
        false_branch: Box::new(minus(
            long_const(s.fixed_rate),
            multiply(long_const(s.one_epoch_reduction), upcast_to_long(epoch)?),
        )),
    });

    let same_script_rule = eq(
        extract_script_bytes(self_box()),
        extract_script_bytes(reward_out.clone()),
    );
    let height_correct = eq(box_creation_height(reward_out.clone())?, height());
    let height_increased = gt(height(), box_creation_height(self_box())?);
    let correct_coins_consumed = eq(
        coins_to_issue,
        minus(
            extract_amount(self_box()),
            extract_amount(reward_out.clone()),
        ),
    );
    let last_coins = le(
        extract_amount(self_box()),
        long_const(s.one_epoch_reduction),
    );
    let outputs_num = eq(size_of(outputs()), int_const(2));

    let correct_miner_output = bool_and(vec![
        eq(
            extract_script_bytes(miner_out.clone()),
            expected_miner_out_script_bytes_val(s.miner_reward_delay, miner_pubkey())?,
        ),
        eq(height(), box_creation_height(miner_out)?),
    ])?;

    let normal_spending = bool_and(vec![
        outputs_num,
        same_script_rule,
        correct_coins_consumed,
        height_correct,
    ])?;

    let prop = to_sigma_prop(bool_and(vec![
        height_increased,
        correct_miner_output,
        bool_or(vec![normal_spending, last_coins])?,
    ])?);

    Ok(ErgoTree::new(ErgoTreeHeader::v0(true), &prop)?)
}

/// Foundation script: controls the treasury box and its gradual depletion.
///
/// The remaining amount decreases according to the emission schedule, and
/// the box can be spent if a custom proposition in register R4 is satisfied.
///
/// Equivalent to JVM `ErgoTreePredef.foundationScript`.
pub fn foundation_script(s: &MonetarySettings) -> Result<ErgoTree, ErgoTreePredefError> {
    let new_foundation_box = by_index(outputs(), int_const(0))?;

    let fir = s.founders_initial_reward;
    let oer = s.one_epoch_reduction;
    let el = i64::from(s.epoch_length);
    let frp = s.fixed_rate_period;
    let frp_minus_1 = frp - 1;

    let full15 = (fir - 2 * oer) * el;
    let full45 = (fir - oer) * el;

    // Nested If/If/If computing the remaining foundation amount at HEIGHT
    let remaining_amount = Expr::If(If {
        condition: Box::new(lt(height(), int_const(frp))),
        true_branch: Box::new(plus(
            long_const(full15 + full45),
            multiply(
                long_const(fir),
                upcast_to_long(minus(int_const(frp_minus_1), height()))?,
            ),
        )),
        false_branch: Box::new(Expr::If(If {
            condition: Box::new(lt(height(), int_const(frp + s.epoch_length))),
            true_branch: Box::new(plus(
                long_const(full15),
                multiply(
                    long_const(fir - oer),
                    upcast_to_long(minus(int_const(frp_minus_1 + s.epoch_length), height()))?,
                ),
            )),
            false_branch: Box::new(Expr::If(If {
                condition: Box::new(lt(height(), int_const(frp + 2 * s.epoch_length))),
                true_branch: Box::new(multiply(
                    long_const(fir - 2 * oer),
                    upcast_to_long(minus(int_const(frp_minus_1 + 2 * s.epoch_length), height()))?,
                )),
                false_branch: Box::new(long_const(0)),
            })),
        })),
    });

    let amount_correct = ge(extract_amount(new_foundation_box.clone()), remaining_amount);
    let same_script_rule = eq(
        extract_script_bytes(self_box()),
        extract_script_bytes(new_foundation_box),
    );
    let custom_proposition = Expr::DeserializeRegister(DeserializeRegister::new(
        RegisterId::NonMandatoryRegisterId(NonMandatoryRegisterId::R4),
        SType::SSigmaProp,
        None,
    )?);

    let prop = Expr::SigmaAnd(SigmaAnd::new(vec![
        to_sigma_prop(amount_correct),
        to_sigma_prop(same_script_rule),
        custom_proposition,
    ])?);

    Ok(ErgoTree::new(ErgoTreeHeader::v0(true), &prop)?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_false_prop_hex() {
        let tree = false_prop().unwrap();
        let hex = tree.to_base16_bytes().unwrap();
        assert_eq!(hex, "10010100d17300");
    }

    #[test]
    fn test_true_prop_hex() {
        let tree = true_prop().unwrap();
        // TrueProp is the same structure as FalseProp but with `true`
        let hex = tree.to_base16_bytes().unwrap();
        // The constant is `true` (0x01) instead of `false` (0x00)
        assert_eq!(hex, "10010101d17300");
    }

    #[test]
    fn test_emission_box_prop_hex() {
        let s = MonetarySettings::default();
        let tree = emission_box_prop(&s).unwrap();
        let hex = tree.to_base16_bytes().unwrap();
        let expected = "101004020e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a7017300730110010204020404040004c0fd4f05808c82f5f6030580b8c9e5ae040580f882ad16040204c0944004c0f407040004000580f882ad16d19683030191a38cc7a7019683020193c2b2a57300007473017302830108cdeeac93a38cc7b2a573030001978302019683040193b1a5730493c2a7c2b2a573050093958fa3730673079973089c73097e9a730a9d99a3730b730c0599c1a7c1b2a5730d00938cc7b2a5730e0001a390c1a7730f";
        assert_eq!(hex, expected);
    }

    #[test]
    fn test_foundation_script_hex() {
        let s = MonetarySettings::default();
        let tree = foundation_script(&s).unwrap();
        let hex = tree.to_base16_bytes().unwrap();
        let expected = "100e040004c094400580809cde91e7b0010580acc7f03704be944004808948058080c7b7e4992c0580b4c4c32104fe884804c0fd4f0580bcc1960b04befd4f05000400ea03d192c1b2a5730000958fa373019a73029c73037e997304a305958fa373059a73069c73077e997308a305958fa373099c730a7e99730ba305730cd193c2a7c2b2a5730d00d5040800";
        assert_eq!(hex, expected);
    }

    #[test]
    fn test_reward_output_script_constructs() {
        // Just verify it constructs without error
        let pk = ProveDlog::new(generator());
        let tree = reward_output_script(720, pk).unwrap();
        assert!(tree.to_base16_bytes().unwrap().starts_with("10"));
    }

    #[test]
    fn test_fee_proposition_constructs() {
        let tree = fee_proposition(720).unwrap();
        assert!(tree.to_base16_bytes().unwrap().starts_with("10"));
    }
}

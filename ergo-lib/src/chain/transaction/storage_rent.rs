//! Storage-rent spending: a port of the storage-rent branch of the JVM's
//! `ErgoInterpreter.verify` and of `ErgoInterpreter.checkExpiredBox`
//! (ergoplatform/ergo v6.0.6,
//! `ergo-wallet/src/main/scala/org/ergoplatform/wallet/interpreter/ErgoInterpreter.scala`).
//! The constants are those of the `Constants` object it imports
//! (`ergo-wallet/src/main/scala/org/ergoplatform/wallet/protocol/Constants.scala`).
//! The arithmetic deliberately reproduces the JVM's 32-bit `Int` semantics.

use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use ergotree_ir::chain::context::Context;
use ergotree_ir::chain::ergo_box::{ErgoBox, RegisterId};
use ergotree_ir::mir::constant::TryExtractInto;
use ergotree_ir::serialization::SigmaSerializable;

use crate::chain::ergo_state_context::ErgoStateContext;

/// The minimum time before a box can be spent via storage rent mechanism
/// (`Constants.StoragePeriod`, `Constants.scala:19`)
pub const STORAGE_PERIOD: u32 = 1051200;
/// What index in ContextExtension the index of output is stored
/// (`Constants.StorageIndexVarId`, `Constants.scala:23`)
pub const STORAGE_EXTENSION_INDEX: u8 = i8::MAX as u8;
/// Cost of a storage-rent spend, in block-cost units
/// (`Constants.StorageContractCost`, `Constants.scala:21`)
pub const STORAGE_CONTRACT_COST: u64 = 50;

/// Outcome of the storage-rent branch for one input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageRentVerdict {
    /// The branch does not apply — the gate is not met, or the JVM's
    /// `recoverWith` fallback fired. Verify the input's script as usual.
    NotApplicable,
    /// The branch applies. The verdict is final: never fall back to the script.
    Verdict(bool),
}

/// Port of the storage-rent branch of `ErgoInterpreter.verify`
/// (`ErgoInterpreter.scala:66-87`).
pub(crate) fn storage_rent_verdict(
    proof: &ProofBytes,
    state_context: &ErgoStateContext,
    context: &Context,
) -> StorageRentVerdict {
    // `proof.length == 0` (`:77`)
    let proof: &[u8] = proof.as_ref();
    if proof.is_empty() {
        rent_verdict_for_empty_proof(state_context, context)
    } else {
        StorageRentVerdict::NotApplicable
    }
}

/// Whether a wallet may spend this input by storage rent with an empty proof:
/// true exactly when `storage_rent_verdict` would return a final `true`.
/// The JVM prover has no rent path; this keeps sigma-rust's signing behaviour.
pub(crate) fn storage_rent_spendable(state_context: &ErgoStateContext, context: &Context) -> bool {
    rent_verdict_for_empty_proof(state_context, context) == StorageRentVerdict::Verdict(true)
}

fn rent_verdict_for_empty_proof(
    state_context: &ErgoStateContext,
    context: &Context,
) -> StorageRentVerdict {
    let height = context.pre_header.height;
    // `hasEnoughTimeToBeSpent` (`:73`) is `Int` subtraction. V1 creation heights
    // may be negative (≥ 2³¹ on the wire), and the difference wraps as the JVM's does.
    let age = (height as i32).wrapping_sub(context.self_box.creation_height as i32);
    if age < STORAGE_PERIOD as i32 {
        return StorageRentVerdict::NotApplicable;
    }
    // `context.extension.values.contains(varId)` (`:77`)
    let var = match context.extension.values.get(&STORAGE_EXTENSION_INDEX) {
        Some(var) => var,
        None => return StorageRentVerdict::NotApplicable,
    };
    // Everything below sits inside `Try { .. }.recoverWith { case _ => super.verify(..) }`
    // (`:78-83`): each failure falls back to ordinary script verification.
    // `.value.asInstanceOf[Short]` (`:79`): only a Short constant passes.
    let idx: i16 = match var.v.clone().try_extract_into() {
        Ok(idx) => idx,
        Err(_) => return StorageRentVerdict::NotApplicable,
    };
    // `spendingTransaction.outputCandidates(idx)` (`:80`): a negative or
    // out-of-range index throws.
    let output = match usize::try_from(idx)
        .ok()
        .and_then(|i| context.outputs.get(i))
    {
        Some(output) => output,
        None => return StorageRentVerdict::NotApplicable,
    };
    let box_bytes_len = match context.self_box.sigma_serialize_bytes() {
        Ok(bytes) => bytes.len(),
        Err(_) => return StorageRentVerdict::NotApplicable,
    };
    // `checkExpiredBox(context.self, outputCandidate, context.preHeader.height)` (`:81`)
    StorageRentVerdict::Verdict(check_expired_box(
        context.self_box,
        output,
        box_bytes_len,
        height,
        state_context.parameters.storage_fee_factor(),
    ))
}

/// Port of `ErgoInterpreter.checkExpiredBox` (`ErgoInterpreter.scala:42-55`).
/// The fee is `Int * Int` (`:43`) and wraps at 32 bits exactly as on the JVM;
/// the value arithmetic is `Long` (wrapping, as on the JVM).
pub(crate) fn check_expired_box(
    self_box: &ErgoBox,
    output: &ErgoBox,
    box_bytes_len: usize,
    current_height: u32,
    storage_fee_factor: i32,
) -> bool {
    // `val storageFee = params.storageFeeFactor * box.bytes.length` (`:43`)
    let storage_fee = i64::from(storage_fee_factor.wrapping_mul(box_bytes_len as i32));
    let box_value = self_box.value.as_i64();
    // `box.value - storageFee <= 0` (`:45`)
    let storage_fee_not_covered = box_value.wrapping_sub(storage_fee) <= 0;
    // `output.creationHeight == currentHeight` (`:46`): equal as Int iff equal as u32
    let correct_creation_height = output.creation_height == current_height;
    // `output.value >= box.value - storageFee` (`:47`)
    let correct_out_value = output.value.as_i64() >= box_value.wrapping_sub(storage_fee);
    // every register except R0 (value) and R3 (reference) preserved, as
    // `box.get(rId) == output.get(rId)` (`:50-52`); deferred like the reference's
    // `lazy val`, since it reads every register
    let correct_registers = || {
        (0..=9u8)
            .map(RegisterId::try_from)
            .map(Result::unwrap)
            .all(|id| {
                id == RegisterId::R0
                    || id == RegisterId::R3
                    || match id {
                        // `ErgoBox.get` returns R4..R9 as stored: a Tuple expression
                        // is its own node (sigmastate `values.scala:807`) and never
                        // equals a Constant (`ConstantNode.equals`, `:356`), so the
                        // stored register values are compared, not their Constants
                        RegisterId::NonMandatoryRegisterId(reg) => {
                            self_box.additional_registers.get(reg)
                                == output.additional_registers.get(reg)
                        }
                        RegisterId::MandatoryRegisterId(_) => {
                            self_box.get_register(id) == output.get_register(id)
                        }
                    }
            })
    };
    // `:54`
    storage_fee_not_covered || (correct_creation_height && correct_out_value && correct_registers())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
pub(crate) mod test_support {
    use super::*;
    use crate::chain::parameters::Parameters;
    use crate::chain::transaction::prover_result::ProverResult;
    use crate::chain::transaction::{Input, Transaction};
    use crate::wallet::tx_context::TransactionContext;
    use ergotree_ir::chain::context::TxIoVec;
    use ergotree_ir::chain::context_extension::ContextExtension;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::{ErgoBoxCandidate, NonMandatoryRegisters};
    use ergotree_ir::chain::tx_id::TxId;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use sigma_test_util::force_any_val;

    /// A small (≈45-byte, so no fee wrap) expired box with a trivially-true script.
    pub(crate) fn expired_box(nano: u64, creation_height: u32, index: u16) -> ErgoBox {
        ErgoBox::new(
            BoxValue::try_from(nano).unwrap(),
            ErgoTree::try_from(Expr::Const(Constant::from(true))).unwrap(),
            None,
            NonMandatoryRegisters::empty(),
            creation_height,
            TxId::zero(),
            index,
        )
        .unwrap()
    }

    /// `self_box` recreated at `height` holding `nano`; script, tokens and registers kept.
    pub(crate) fn recreated(self_box: &ErgoBox, nano: u64, height: u32) -> ErgoBox {
        let mut out = self_box.clone();
        out.value = BoxValue::try_from(nano).unwrap();
        out.creation_height = height;
        out
    }

    /// A tx spending `boxes` (all with empty proofs; `var127[i] = Some(idx)` puts a
    /// Short in variable 127 of input i) into `outs`, validated at `height` under
    /// default parameters (storageFeeFactor 1_250_000).
    pub(crate) fn tx_spending(
        boxes: &[ErgoBox],
        var127: &[Option<i16>],
        outs: &[ErgoBox],
        height: u32,
    ) -> (TransactionContext<Transaction>, ErgoStateContext) {
        let inputs: Vec<Input> = boxes
            .iter()
            .zip(var127)
            .map(|(b, v)| {
                let mut ext = ContextExtension::empty();
                if let Some(idx) = v {
                    ext.values
                        .insert(STORAGE_EXTENSION_INDEX, Constant::from(*idx));
                }
                Input::new(
                    b.box_id(),
                    ProverResult {
                        proof: ProofBytes::Empty,
                        extension: ext,
                    },
                )
            })
            .collect();
        let candidates: Vec<ErgoBoxCandidate> = outs
            .iter()
            .map(|o| ErgoBoxCandidate {
                value: o.value,
                ergo_tree: o.ergo_tree.clone(),
                tokens: o.tokens.clone(),
                additional_registers: o.additional_registers.clone(),
                creation_height: o.creation_height,
            })
            .collect();
        let tx = Transaction::new(
            TxIoVec::from_vec(inputs).unwrap(),
            None,
            TxIoVec::from_vec(candidates).unwrap(),
        )
        .unwrap();
        let tx_ctx = TransactionContext::new(tx, boxes.to_vec(), vec![]).unwrap();
        let mut state_context = force_any_val::<ErgoStateContext>();
        state_context.parameters = Parameters::default();
        state_context.pre_header.height = height;
        (tx_ctx, state_context)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::chain::parameters::Parameters;
    use crate::chain::transaction::ergo_transaction::TxValidationError;
    use crate::chain::transaction::verify_tx_input_proof;
    use crate::wallet::signing::make_context;
    use ergotree_ir::chain::context_extension::ContextExtension;
    use ergotree_ir::chain::ergo_box::box_value::BoxValue;
    use ergotree_ir::chain::ergo_box::{
        ErgoBoxCandidate, EvaluatedTuple, NonMandatoryRegisterId, NonMandatoryRegisters,
        RegisterValue,
    };
    use ergotree_ir::chain::tx_id::TxId;
    use ergotree_ir::ergo_tree::ErgoTree;
    use ergotree_ir::mir::constant::Constant;
    use ergotree_ir::mir::expr::Expr;
    use ergotree_ir::mir::tuple::Tuple;
    use sigma_test_util::force_any_val;

    /// Live mainnet storageFeeFactor, and `Parameters::default()`.
    const FACTOR: i32 = 1_250_000;
    const H: u32 = 2_000_000;

    // ---- check_expired_box: the JVM's Int arithmetic ----

    #[test]
    fn dust_box_is_spendable_regardless_of_output() {
        // 1717 bytes: fee 2_146_250_000, no wrap. value == fee → `value - fee <= 0`.
        let b = expired_box(2_146_250_000, 0, 0);
        let junk = recreated(&b, BoxValue::MIN_RAW, 7);
        assert!(check_expired_box(&b, &junk, 1717, H, FACTOR));
    }

    #[test]
    fn fee_wraps_negative_at_1718_bytes() {
        // 1718 * 1_250_000 = 2_147_500_000 → Int wraps to -2_147_467_296.
        let b = expired_box(2_000_000_000, 0, 0);
        // True-fee math would call this dust; the JVM does not.
        assert!(!check_expired_box(
            &b,
            &recreated(&b, BoxValue::MIN_RAW, H),
            1718,
            H,
            FACTOR
        ));
        // The JVM demands value + 2_147_467_296 in the recreated box.
        let need = 2_000_000_000 + 2_147_467_296;
        assert!(!check_expired_box(
            &b,
            &recreated(&b, need - 1, H),
            1718,
            H,
            FACTOR
        ));
        assert!(check_expired_box(
            &b,
            &recreated(&b, need, H),
            1718,
            H,
            FACTOR
        ));
    }

    #[test]
    fn fee_wraps_small_positive_at_3436_bytes() {
        // 3436 * 1_250_000 = 4_295_000_000 → Int wraps to 32_704.
        let b = expired_box(5_000_000_000, 0, 0);
        assert!(check_expired_box(
            &b,
            &recreated(&b, 5_000_000_000 - 32_704, H),
            3436,
            H,
            FACTOR
        ));
        assert!(!check_expired_box(
            &b,
            &recreated(&b, 5_000_000_000 - 32_705, H),
            3436,
            H,
            FACTOR
        ));
    }

    #[test]
    fn recreation_needs_current_height_and_preserved_registers() {
        let b = expired_box(5_000_000_000, 0, 0);
        let fee = 125_000_000u64; // 100 bytes * 1_250_000
        assert!(check_expired_box(
            &b,
            &recreated(&b, 5_000_000_000 - fee, H),
            100,
            H,
            FACTOR
        ));
        assert!(!check_expired_box(
            &b,
            &recreated(&b, 5_000_000_000 - fee, H - 1),
            100,
            H,
            FACTOR
        ));
        let mut other_script = recreated(&b, 5_000_000_000 - fee, H);
        other_script.ergo_tree = ErgoTree::try_from(Expr::Const(Constant::from(false))).unwrap();
        assert!(!check_expired_box(&b, &other_script, 100, H, FACTOR));
    }

    // ---- check_expired_box: R4..R9 compared as `ErgoBox.get` values ----

    /// `b` with its non-mandatory registers replaced by `regs`.
    fn with_regs(b: &ErgoBox, regs: NonMandatoryRegisters) -> ErgoBox {
        ErgoBox::new(
            b.value,
            b.ergo_tree.clone(),
            b.tokens.clone(),
            regs,
            b.creation_height,
            TxId::zero(),
            0,
        )
        .unwrap()
    }

    /// The value `(1, 2)` as a register holding a Tuple expression, and as one
    /// holding the equal tuple Constant.
    fn tuple_expr_and_constant() -> (RegisterValue, RegisterValue) {
        let tuple = Tuple::new(vec![Expr::Const(1i32.into()), Expr::Const(2i32.into())]).unwrap();
        let et = EvaluatedTuple::new(tuple).unwrap();
        let constant = et.as_constant().clone();
        (
            RegisterValue::ParsedTupleExpr(et),
            RegisterValue::Parsed(constant),
        )
    }

    #[test]
    fn tuple_expr_register_is_not_preserved_by_the_equal_tuple_constant() {
        // JVM: the input's R4 is a `Tuple` node, the output's a `ConstantNode`, and
        // they are never equal (`values.scala:356`, `:807`).
        let (expr, constant) = tuple_expr_and_constant();
        let regs = NonMandatoryRegisters::try_from(vec![expr]).unwrap();
        let self_box = with_regs(&expired_box(5_000_000_000, 0, 0), regs);
        // Both encodings survive the wire: the input's R4 parses back as a Tuple
        // expression ...
        let self_box =
            ErgoBox::sigma_parse_bytes(&self_box.sigma_serialize_bytes().unwrap()).unwrap();
        assert!(matches!(
            self_box
                .additional_registers
                .get(NonMandatoryRegisterId::R4),
            Some(RegisterValue::ParsedTupleExpr(_))
        ));
        // ... and the output's as a Constant.
        let mut out = recreated(&self_box, 5_000_000_000, H);
        out.additional_registers = NonMandatoryRegisters::try_from(vec![constant]).unwrap();
        let out_bytes = ErgoBoxCandidate::from(out).sigma_serialize_bytes().unwrap();
        let out_candidate = ErgoBoxCandidate::sigma_parse_bytes(&out_bytes).unwrap();
        let out = ErgoBox::from_box_candidate(&out_candidate, TxId::zero(), 0).unwrap();
        assert!(matches!(
            out.additional_registers.get(NonMandatoryRegisterId::R4),
            Some(RegisterValue::Parsed(_))
        ));
        assert!(!check_expired_box(&self_box, &out, 100, H, FACTOR));
    }

    #[test]
    fn same_tuple_expr_register_is_preserved() {
        let (expr, _) = tuple_expr_and_constant();
        let regs = NonMandatoryRegisters::try_from(vec![expr]).unwrap();
        let self_box = with_regs(&expired_box(5_000_000_000, 0, 0), regs);
        let out = recreated(&self_box, 5_000_000_000, H);
        assert!(check_expired_box(&self_box, &out, 100, H, FACTOR));
    }

    #[test]
    fn same_constant_register_is_preserved() {
        let (_, constant) = tuple_expr_and_constant();
        let regs = NonMandatoryRegisters::try_from(vec![constant]).unwrap();
        let self_box = with_regs(&expired_box(5_000_000_000, 0, 0), regs);
        let out = recreated(&self_box, 5_000_000_000, H);
        assert!(check_expired_box(&self_box, &out, 100, H, FACTOR));
    }

    // ---- the gate and the fallback arms ----

    /// Runs `f` on a validation context spending `self_box` at `height`, with
    /// variable 127 = `var127`.
    fn with_ctx<T>(
        self_box: &ErgoBox,
        outputs: &[ErgoBox],
        var127: Option<Constant>,
        height: u32,
        f: impl FnOnce(&ErgoStateContext, &Context) -> T,
    ) -> T {
        let mut ext = ContextExtension::empty();
        if let Some(c) = var127 {
            ext.values.insert(STORAGE_EXTENSION_INDEX, c);
        }
        let base = force_any_val::<Context>();
        let mut pre_header = base.pre_header.clone();
        pre_header.height = height;
        let ctx = Context {
            height,
            self_box,
            outputs,
            extension: &ext,
            pre_header,
            ..base
        };
        let mut state_context = force_any_val::<ErgoStateContext>();
        state_context.parameters = Parameters::default();
        state_context.pre_header.height = height;
        f(&state_context, &ctx)
    }

    fn verdict(
        b: &ErgoBox,
        outs: &[ErgoBox],
        var127: Option<Constant>,
        h: u32,
    ) -> StorageRentVerdict {
        with_ctx(b, outs, var127, h, |s, c| {
            storage_rent_verdict(&ProofBytes::Empty, s, c)
        })
    }

    #[test]
    fn gate_requires_storage_period_age() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let out = [recreated(&b, 5_000_000_000, p)];
        assert_eq!(
            verdict(&b, &out, Some(Constant::from(0i16)), p),
            StorageRentVerdict::Verdict(true)
        );
        let out = [recreated(&b, 5_000_000_000, p - 1)];
        assert_eq!(
            verdict(&b, &out, Some(Constant::from(0i16)), p - 1),
            StorageRentVerdict::NotApplicable
        );
    }

    #[test]
    fn gate_uses_signed_age_for_v1_negative_creation_heights() {
        // creation height 0xFFFF_FFFF is Int -1 on the JVM: age = height + 1.
        let b = expired_box(5_000_000_000, u32::MAX, 0);
        let h = STORAGE_PERIOD - 1;
        let out = [recreated(&b, 5_000_000_000, h)];
        assert_eq!(
            verdict(&b, &out, Some(Constant::from(0i16)), h),
            StorageRentVerdict::Verdict(true)
        );
    }

    #[test]
    fn non_empty_proof_is_not_a_rent_spend() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let out = [recreated(&b, 5_000_000_000, p)];
        let v = with_ctx(&b, &out, Some(Constant::from(0i16)), p, |s, c| {
            storage_rent_verdict(&ProofBytes::Some(vec![1]), s, c)
        });
        assert_eq!(v, StorageRentVerdict::NotApplicable);
    }

    #[test]
    fn missing_var_127_is_not_a_rent_spend() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let out = [recreated(&b, 5_000_000_000, p)];
        assert_eq!(
            verdict(&b, &out, None, p),
            StorageRentVerdict::NotApplicable
        );
    }

    #[test]
    fn var_127_that_is_not_a_short_falls_back_to_the_script() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let out = [recreated(&b, 5_000_000_000, p)];
        for c in [
            Constant::from(0i32),
            Constant::from(0i8),
            Constant::from(0i64),
        ] {
            assert_eq!(
                verdict(&b, &out, Some(c), p),
                StorageRentVerdict::NotApplicable
            );
        }
    }

    #[test]
    fn negative_or_out_of_range_index_falls_back_to_the_script() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let out = [recreated(&b, 5_000_000_000, p)]; // one output
        for idx in [-1i16, 1, i16::MAX] {
            assert_eq!(
                verdict(&b, &out, Some(Constant::from(idx)), p),
                StorageRentVerdict::NotApplicable,
                "idx {idx}"
            );
        }
    }

    #[test]
    fn failed_recreation_is_a_final_false() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let out = [recreated(&b, 5_000_000_000, p - 1)]; // wrong creation height
        assert_eq!(
            verdict(&b, &out, Some(Constant::from(0i16)), p),
            StorageRentVerdict::Verdict(false)
        );
    }

    #[test]
    fn spendable_is_exactly_a_final_true() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let good = [recreated(&b, 5_000_000_000, p)];
        let bad = [recreated(&b, 5_000_000_000, p - 1)];
        assert!(with_ctx(
            &b,
            &good,
            Some(Constant::from(0i16)),
            p,
            storage_rent_spendable
        ));
        assert!(!with_ctx(
            &b,
            &bad,
            Some(Constant::from(0i16)),
            p,
            storage_rent_spendable
        ));
        assert!(!with_ctx(&b, &good, None, p, storage_rent_spendable));
    }

    // ---- the call sites ----

    #[test]
    fn verify_tx_input_proof_rent_arm_is_final_and_costs_50() {
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        for (out_height, expected) in [(p, true), (p - 1, false)] {
            let (tx_ctx, sc) = tx_spending(
                std::slice::from_ref(&b),
                &[Some(0)],
                &[recreated(&b, 5_000_000_000, out_height)],
                p,
            );
            let mut ctx = make_context(&sc, &tx_ctx, 0).unwrap();
            let bytes = tx_ctx.spending_tx.bytes_to_sign().unwrap();
            let res = verify_tx_input_proof(&tx_ctx, &mut ctx, &sc, 0, &bytes).unwrap();
            assert_eq!(
                (res.result, res.cost),
                (expected, STORAGE_CONTRACT_COST),
                "out height {out_height}"
            );
        }
    }

    #[test]
    fn validate_rejects_a_failed_rent_verdict_even_for_a_trivially_true_script() {
        // ERG preserved (5 ERG in, 5 ERG out); the recreation has the wrong height.
        // The rent check fails, and the trivially-true script must NOT rescue it.
        let b = expired_box(5_000_000_000, 0, 0);
        let p = STORAGE_PERIOD;
        let (tx_ctx, sc) = tx_spending(
            std::slice::from_ref(&b),
            &[Some(0)],
            &[recreated(&b, 5_000_000_000, p - 1)],
            p,
        );
        let res = tx_ctx.validate(&sc);
        assert!(
            matches!(res, Err(TxValidationError::ReducedToFalse(0, _))),
            "{res:?}"
        );
    }
}

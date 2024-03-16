use ergotree_interpreter::{eval::context::Context, sigma_protocol::prover::ProofBytes};
use ergotree_ir::{
    chain::ergo_box::RegisterId, mir::constant::TryExtractInto, serialization::SigmaSerializable,
};

use crate::chain::ergo_state_context::ErgoStateContext;

use super::Input;

/// The minimum time before a box can be spent via storage rent mechanism
pub const STORAGE_PERIOD: u32 = 1051200;
/// What index in ContextExtension the index of output is stored.
pub const STORAGE_EXTENSION_INDEX: u8 = i8::MAX as u8;

// Attempt to spend a box with storage rent. Returns None if any of the required conditions is not met
pub(crate) fn try_spend_storage_rent(
    input_box: &Input,
    state_context: &ErgoStateContext,
    context: &Context,
) -> Option<()> {
    if context
        .pre_header
        .height
        .checked_sub(context.self_box.creation_height)?
        >= STORAGE_PERIOD
        && matches!(input_box.spending_proof.proof, ProofBytes::Empty)
    {
        let output_idx: i16 = context
            .extension
            .values
            .get(&STORAGE_EXTENSION_INDEX)?
            .v
            .clone()
            .try_extract_into()
            .ok()?;
        let output_candidate = context.outputs.get(output_idx as usize)?;

        let storage_fee = output_candidate.sigma_serialize_bytes().ok()?.len() as u64
            * state_context.parameters.storage_fee_factor() as u64;
        // If the box's value is less than the required storage fee, the box can be spent without any further restrictions
        if context.self_box.value.as_u64() <= &storage_fee {
            return Some(());
        }
        if output_candidate.creation_height != state_context.pre_header.height
            || *output_candidate.value.as_u64() < context.self_box.value.as_u64() - storage_fee
        {
            return None;
        }
        // Require that all registers except value (R0) and creation info (R3) be preserved
        let registers_preserved =
            (0..=9u8)
                .map(RegisterId::try_from)
                .map(Result::unwrap)
                .all(|id| {
                    id == RegisterId::R0
                        || id == RegisterId::R3
                        || context.self_box.get_register(id) == output_candidate.get_register(id)
                });
        if registers_preserved {
            return Some(());
        }
    }
    None
}

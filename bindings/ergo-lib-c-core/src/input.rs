//! Ergo input

use ergo_lib::{chain, ergotree_interpreter::sigma_protocol::prover::ProofBytes};

use crate::{
    context_extension::{ContextExtension, ContextExtensionPtr},
    ergo_box::{BoxId, BoxIdPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// Unsigned inputs used in constructing unsigned transactions
#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedInput(pub chain::transaction::UnsignedInput);
pub type UnsignedInputPtr = *mut UnsignedInput;
pub type ConstUnsignedInputPtr = *const UnsignedInput;

pub unsafe fn unsigned_input_box_id(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    box_id_out: *mut BoxIdPtr,
) -> Result<(), Error> {
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    let unsigned_input = const_ptr_as_ref(unsigned_input_ptr, "unsigned_input_ptr")?;
    let box_id = BoxId(unsigned_input.0.box_id.clone());
    *box_id_out = Box::into_raw(Box::new(box_id));
    Ok(())
}

pub unsafe fn unsigned_input_context_extension(
    unsigned_input_ptr: ConstUnsignedInputPtr,
    context_extension_out: *mut ContextExtensionPtr,
) -> Result<(), Error> {
    let context_extension_out = mut_ptr_as_mut(context_extension_out, "context_extension_out")?;
    let unsigned_input = const_ptr_as_ref(unsigned_input_ptr, "unsigned_input_ptr")?;
    let context_extension = ContextExtension(unsigned_input.0.extension.clone());
    *context_extension_out = Box::into_raw(Box::new(context_extension));
    Ok(())
}

/// Signed inputs used in signed transactions
#[derive(PartialEq, Debug, Clone)]
pub struct Input(pub(crate) chain::transaction::Input);
pub type InputPtr = *mut Input;
pub type ConstInputPtr = *const Input;

pub unsafe fn input_box_id(
    input_ptr: ConstInputPtr,
    box_id_out: *mut BoxIdPtr,
) -> Result<(), Error> {
    let box_id_out = mut_ptr_as_mut(box_id_out, "box_id_out")?;
    let input = const_ptr_as_ref(input_ptr, "input_ptr")?;
    let box_id = BoxId(input.0.box_id.clone());
    *box_id_out = Box::into_raw(Box::new(box_id));
    Ok(())
}

pub unsafe fn input_spending_proof(
    input_ptr: ConstInputPtr,
    prover_result_out: *mut ProverResultPtr,
) -> Result<(), Error> {
    let prover_result_out = mut_ptr_as_mut(prover_result_out, "prover_result_out")?;
    let input = const_ptr_as_ref(input_ptr, "input_ptr")?;
    let prover_result = ProverResult(input.0.spending_proof.clone());
    *prover_result_out = Box::into_raw(Box::new(prover_result));
    Ok(())
}

/// Proof of correctness of tx spending
#[derive(PartialEq, Debug, Clone)]
pub struct ProverResult(chain::transaction::input::prover_result::ProverResult);
pub type ProverResultPtr = *mut ProverResult;
pub type ConstProverResultPtr = *const ProverResult;

pub unsafe fn prover_result_proof_len(
    prover_result_ptr: ConstProverResultPtr,
) -> Result<usize, Error> {
    let prover_result = const_ptr_as_ref(prover_result_ptr, "prover_result_ptr")?;
    Ok(match &prover_result.0.proof {
        ProofBytes::Some(b) => b.len(),
        ProofBytes::Empty => 0,
    })
}

pub unsafe fn prover_result_proof(
    prover_result_ptr: ConstProverResultPtr,
    output: *mut u8,
) -> Result<(), Error> {
    let prover_result = const_ptr_as_ref(prover_result_ptr, "prover_result_ptr")?;
    let src: Vec<_> = prover_result.0.proof.clone().into();
    std::ptr::copy_nonoverlapping(src.as_ptr(), output, src.len());
    Ok(())
}

pub unsafe fn prover_result_context_extension(
    prover_result_ptr: ConstProverResultPtr,
    context_extension_out: *mut ContextExtensionPtr,
) -> Result<(), Error> {
    let prover_result = const_ptr_as_ref(prover_result_ptr, "prover_result_ptr")?;
    let context_extension_out = mut_ptr_as_mut(context_extension_out, "context_extension_out")?;
    *context_extension_out = Box::into_raw(Box::new(ContextExtension(
        prover_result.0.extension.clone(),
    )));
    Ok(())
}

pub unsafe fn prover_result_to_json(
    prover_result_ptr: ConstProverResultPtr,
) -> Result<String, Error> {
    let prover_result = const_ptr_as_ref(prover_result_ptr, "prover_result_ptr")?;
    let s = serde_json::to_string_pretty(&prover_result.0.clone())?;
    Ok(s)
}

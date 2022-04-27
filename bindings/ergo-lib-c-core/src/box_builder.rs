use ergo_lib::chain;

use crate::{
    constant::{ConstConstantPtr, Constant, ConstantPtr},
    contract::ConstContractPtr,
    ergo_box::{
        BoxValue, BoxValuePtr, ConstBoxValuePtr, ErgoBoxCandidate, ErgoBoxCandidatePtr,
        NonMandatoryRegisterId,
    },
    token::{ConstTokenAmountPtr, ConstTokenIdPtr, ConstTokenPtr},
    util::{const_ptr_as_ref, mut_ptr_as_mut},
    Error,
};

/// ErgoBoxCandidate builder
pub struct ErgoBoxCandidateBuilder(chain::ergo_box::box_builder::ErgoBoxCandidateBuilder);
pub type ErgoBoxCandidateBuilderPtr = *mut ErgoBoxCandidateBuilder;
pub type ConstErgoBoxCandidateBuilderPtr = *const ErgoBoxCandidateBuilder;

/// Create builder with required box parameters:
/// `value` - amount of money associated with the box
/// `contract` - guarding contract, which should be evaluated to true in order
/// to open(spend) this box
/// `creation_height` - height when a transaction containing the box is created.
/// It should not exceed height of the block, containing the transaction with this box.
pub unsafe fn ergo_box_candidate_builder_new(
    value_ptr: ConstBoxValuePtr,
    contract_ptr: ConstContractPtr,
    creation_height: u32,
    builder_out: *mut ErgoBoxCandidateBuilderPtr,
) -> Result<(), Error> {
    let value = const_ptr_as_ref(value_ptr, "value_ptr")?;
    let contract = const_ptr_as_ref(contract_ptr, "contract_ptr")?;
    let builder_out = mut_ptr_as_mut(builder_out, "builder_out")?;
    *builder_out = Box::into_raw(Box::new(ErgoBoxCandidateBuilder(
        chain::ergo_box::box_builder::ErgoBoxCandidateBuilder::new(
            value.0,
            contract.0.ergo_tree(),
            creation_height,
        ),
    )));
    Ok(())
}

/// Set minimal value (per byte of the serialized box size)
pub unsafe fn ergo_box_candidate_builder_set_min_box_value_per_byte(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    new_min_value_per_byte: u32,
) -> Result<(), Error> {
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    builder_mut
        .0
        .set_min_box_value_per_byte(new_min_value_per_byte);
    Ok(())
}

/// Get minimal value (per byte of the serialized box size)
pub unsafe fn ergo_box_candidate_builder_min_box_value_per_byte(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
) -> Result<u32, Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    Ok(builder.0.min_box_value_per_byte())
}

/// Set new box value
pub unsafe fn ergo_box_candidate_builder_set_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    value_ptr: ConstBoxValuePtr,
) -> Result<(), Error> {
    let value = const_ptr_as_ref(value_ptr, "value_ptr")?;
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    builder_mut.0.set_value(value.0);
    Ok(())
}

/// Get box value
pub unsafe fn ergo_box_candidate_builder_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    let value_out = mut_ptr_as_mut(value_out, "value_out")?;
    *value_out = Box::into_raw(Box::new(BoxValue(*builder.0.value())));
    Ok(())
}

/// Calculate serialized box size(in bytes)
pub unsafe fn ergo_box_candidate_builder_calc_box_size_bytes(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
) -> Result<usize, Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    let b = builder.0.calc_box_size_bytes()?;
    Ok(b)
}

/// Calculate minimal box value for the current box serialized size(in bytes)
pub unsafe fn ergo_box_candidate_builder_calc_min_box_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    let value_out = mut_ptr_as_mut(value_out, "value_out")?;
    let value = builder.0.calc_min_box_value().map(BoxValue)?;
    *value_out = Box::into_raw(Box::new(value));
    Ok(())
}

/// Set register with a given id (R4-R9) to the given value
pub unsafe fn ergo_box_candidate_builder_set_register_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
    constant_ptr: ConstConstantPtr,
) -> Result<(), Error> {
    let constant = const_ptr_as_ref(constant_ptr, "constant_ptr")?;
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    builder_mut
        .0
        .set_register_value(register_id.into(), constant.0.clone());
    Ok(())
}

/// Returns register value for the given register id (R4-R9), or None if the register is empty
pub unsafe fn ergo_box_candidate_builder_register_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
    constant_out: *mut ConstantPtr,
) -> Result<bool, Error> {
    let constant_out = mut_ptr_as_mut(constant_out, "constant_out")?;
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    match builder.0.register_value(&register_id.into()) {
        Some(value) => {
            *constant_out = Box::into_raw(Box::new(Constant(value.clone())));
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Delete register value(make register empty) for the given register id (R4-R9)
pub unsafe fn ergo_box_candidate_builder_delete_register_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
) -> Result<(), Error> {
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    builder_mut.0.delete_register_value(&register_id.into());
    Ok(())
}

/// Mint token, as defined in <https://github.com/ergoplatform/eips/blob/master/eip-0004.md>
/// `token` - token id(box id of the first input box in transaction) and token amount,
/// `token_name` - token name (will be encoded in R4),
/// `token_desc` - token description (will be encoded in R5),
/// `num_decimals` - number of decimals (will be encoded in R6)
pub unsafe fn ergo_box_candidate_builder_mint_token(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    token_ptr: ConstTokenPtr,
    token_name: &str,
    token_desc: &str,
    num_decimals: usize,
) -> Result<(), Error> {
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    let token = const_ptr_as_ref(token_ptr, "token_ptr")?;
    builder_mut.0.mint_token(
        token.0.clone(),
        token_name.into(),
        token_desc.into(),
        num_decimals,
    );
    Ok(())
}

/// Add given token id and token amount
pub unsafe fn ergo_box_candidate_builder_add_token(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    token_id_ptr: ConstTokenIdPtr,
    token_amount_ptr: ConstTokenAmountPtr,
) -> Result<(), Error> {
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    let token_id = const_ptr_as_ref(token_id_ptr, "token_id_ptr")?;
    let token_amount = const_ptr_as_ref(token_amount_ptr, "token_amount_ptr")?;
    builder_mut
        .0
        .add_token(ergo_lib::ergotree_ir::chain::token::Token {
            token_id: token_id.0.clone(),
            amount: token_amount.0,
        });
    Ok(())
}

/// Build the box candidate
pub unsafe fn ergo_box_candidate_builder_build(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    ergo_box_candidate_out: *mut ErgoBoxCandidatePtr,
) -> Result<(), Error> {
    let ergo_box_candidate_out = mut_ptr_as_mut(ergo_box_candidate_out, "ergo_box_candidate_out")?;
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    let candidate = builder.0.clone().build().map(ErgoBoxCandidate)?;
    *ergo_box_candidate_out = Box::into_raw(Box::new(candidate));
    Ok(())
}

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

pub unsafe fn ergo_box_candidate_builder_min_box_value_per_byte(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
) -> Result<u32, Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    Ok(builder.0.min_box_value_per_byte())
}

pub unsafe fn ergo_box_candidate_builder_set_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    value_ptr: ConstBoxValuePtr,
) -> Result<(), Error> {
    let value = const_ptr_as_ref(value_ptr, "value_ptr")?;
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    builder_mut.0.set_value(value.0);
    Ok(())
}

pub unsafe fn ergo_box_candidate_builder_value(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
    value_out: *mut BoxValuePtr,
) -> Result<(), Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    let value_out = mut_ptr_as_mut(value_out, "value_out")?;
    *value_out = Box::into_raw(Box::new(BoxValue(*builder.0.value())));
    Ok(())
}

pub unsafe fn ergo_box_candidate_builder_calc_box_size_bytes(
    builder_ptr: ConstErgoBoxCandidateBuilderPtr,
) -> Result<usize, Error> {
    let builder = const_ptr_as_ref(builder_ptr, "builder_ptr")?;
    let b = builder.0.calc_box_size_bytes()?;
    Ok(b)
}

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

pub unsafe fn ergo_box_candidate_builder_delete_register_value(
    builder_mut: ErgoBoxCandidateBuilderPtr,
    register_id: NonMandatoryRegisterId,
) -> Result<(), Error> {
    let builder_mut = mut_ptr_as_mut(builder_mut, "builder_mut")?;
    builder_mut.0.delete_register_value(&register_id.into());
    Ok(())
}

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

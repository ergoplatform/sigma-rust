use ergo_lib::{
    ergotree_ir::chain::ergo_box::box_value::BoxValue,
    wallet::tx_builder::TxBuilder as TxBuilderInner,
};
use pyo3::prelude::*;

use crate::{
    chain::{
        address::Address,
        context_extension::ContextExtension,
        ergo_box::{BoxId, ErgoBoxCandidate},
        token::Token,
    },
    to_value_error,
    wallet::box_selector::BoxSelection,
};

use super::{data_input::DataInput, UnsignedTransaction};

#[pyclass(from_py_object)]
#[derive(Clone)]
pub(crate) struct TxBuilder(TxBuilderInner<ergo_lib::ergotree_ir::chain::ergo_box::ErgoBox>);

#[pymethods]
impl TxBuilder {
    #[new]
    fn new(
        box_selection: &BoxSelection,
        output_candidates: Vec<ErgoBoxCandidate>,
        current_height: u32,
        fee_amount: u64,
        change_address: &Address,
    ) -> PyResult<Self> {
        let output_candidates = output_candidates.into_iter().map(Into::into).collect();
        Ok(TxBuilder(TxBuilderInner::new(
            box_selection.clone().into(),
            output_candidates,
            current_height,
            BoxValue::new(fee_amount).map_err(to_value_error)?,
            change_address.clone().into(),
        )))
    }
    fn set_data_inputs(&mut self, data_inputs: Vec<DataInput>) {
        self.0
            .set_data_inputs(data_inputs.into_iter().map(Into::into).collect());
    }
    fn set_context_extension(&mut self, box_id: BoxId, context_extension: ContextExtension) {
        self.0
            .set_context_extension(box_id.into(), context_extension.into());
    }
    pub fn set_token_burn_permit(&mut self, tokens: Vec<Token>) {
        self.0
            .set_token_burn_permit(tokens.into_iter().map(|t| t.into()).collect())
    }
    fn build(&self) -> PyResult<UnsignedTransaction> {
        self.0
            .clone()
            .build()
            .map(Into::into)
            .map_err(to_value_error)
    }
}

use crate::{
    chain::{ergo_box::ErgoBox, token::Token},
    to_value_error,
};
use derive_more::{From, Into};
use ergo_lib::{
    ergotree_ir::chain::ergo_box::{box_value::BoxValue, BoxTokens, ErgoBox as ErgoBoxInner},
    wallet::box_selector::{
        BoxSelection as BoxSelectionInner, BoxSelector,
        ErgoBoxAssetsData as ErgoBoxAssetsDataInner, SimpleBoxSelector,
    },
};
use pyo3::prelude::*;

#[pyclass(eq, frozen, from_py_object)]
#[derive(Clone, PartialEq, Eq, From, Into)]
pub(crate) struct ErgoBoxAssetsData(ErgoBoxAssetsDataInner);

#[pymethods]
impl ErgoBoxAssetsData {
    #[new]
    fn new(value: u64, tokens: Vec<Token>) -> PyResult<Self> {
        let tokens = if tokens.is_empty() {
            None
        } else {
            Some(
                BoxTokens::from_vec(tokens.into_iter().map(Into::into).collect())
                    .map_err(to_value_error)?,
            )
        };
        Ok(Self(ErgoBoxAssetsDataInner {
            value: BoxValue::new(value).map_err(to_value_error)?,
            tokens,
        }))
    }
    #[getter]
    fn value(&self) -> u64 {
        self.0.value.into()
    }
    #[getter]
    fn tokens(&self) -> Vec<Token> {
        self.0
            .tokens
            .as_ref()
            .into_iter()
            .flatten()
            .copied()
            .map(Into::into)
            .collect()
    }
}

#[pyclass(eq, from_py_object)]
#[derive(Clone, PartialEq, Eq, From, Into)]
pub struct BoxSelection(BoxSelectionInner<ErgoBoxInner>);

#[pymethods]
impl BoxSelection {
    #[new]
    fn new(boxes: Vec<ErgoBox>, change_boxes: Vec<ErgoBoxAssetsData>) -> PyResult<Self> {
        Ok(Self(BoxSelectionInner {
            boxes: boxes
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(to_value_error)?,
            change_boxes: change_boxes.into_iter().map(Into::into).collect::<Vec<_>>(),
        }))
    }
    #[getter]
    fn boxes(&self) -> Vec<ErgoBox> {
        self.0.boxes.iter().cloned().map(Into::into).collect()
    }
    #[getter]
    fn change_boxes(&self) -> Vec<ErgoBoxAssetsData> {
        self.0
            .change_boxes
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

/// Select boxes whose value and tokens sum to target_balance and target_tokens.
/// This uses a simple strategy where boxes are sorted by token amounts and selected in descending order
#[pyfunction]
pub(crate) fn select_boxes_simple(
    inputs: Vec<ErgoBox>,
    target_balance: u64,
    target_tokens: Vec<Token>,
) -> PyResult<BoxSelection> {
    // TODO: use bytemuck to convert collections of newtypes into inner type with zero-cost
    let target_tokens: Vec<_> = target_tokens.into_iter().map(|t| t.0).collect();
    SimpleBoxSelector::new()
        .select(
            inputs.into_iter().map(|p| p.0).collect(),
            BoxValue::new(target_balance).map_err(to_value_error)?,
            &target_tokens,
        )
        .map(Into::into)
        .map_err(to_value_error)
}

//! DataInput type

use ergo_lib::chain;
use wasm_bindgen::prelude::*;
use crate::ergo_box::BoxId;

extern crate derive_more;
use derive_more::{From, Into};

/// Inputs, that are used to enrich script context, but won't be spent by the transaction
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct DataInput(chain::transaction::DataInput);

#[wasm_bindgen]
impl DataInput {
    /// Get box id
    pub fn box_id(&self) -> BoxId {
        self.0.box_id.clone().into()
    }
}

/// DataInput collection
#[wasm_bindgen]
pub struct DataInputs(Vec<DataInput>);

#[wasm_bindgen]
impl DataInputs {
    /// Create empty DataInputs
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        DataInputs(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> DataInput {
        self.0[index].clone()
    }

    /// Adds an elements to the collection
    pub fn add(&mut self, elem: &DataInput) {
        self.0.push(elem.clone());
    }
}

impl From<&DataInputs> for Vec<chain::transaction::DataInput> {
    fn from(v: &DataInputs) -> Self {
        v.0.clone().iter().map(|i| i.0.clone()).collect()
    }
}
impl From<Vec<chain::transaction::DataInput>> for DataInputs {
    fn from(v: Vec<chain::transaction::DataInput>) -> Self {
        DataInputs(v.into_iter().map(DataInput::from).collect())
    }
}

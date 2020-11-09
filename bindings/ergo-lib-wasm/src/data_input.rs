//! DataInput type

use ergo_lib::chain;
use wasm_bindgen::prelude::*;

/// Inputs, that are used to enrich script context, but won't be spent by the transaction
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DataInput(chain::transaction::DataInput);

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

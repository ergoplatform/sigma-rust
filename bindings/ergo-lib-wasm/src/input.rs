//! Ergo input

use crate::context_extension::ContextExtension;
use crate::ergo_box::BoxId;
use crate::prover_result;
use ergo_lib::chain;
use wasm_bindgen::prelude::*;

extern crate derive_more;
use derive_more::{From, Into};

/// Unsigned inputs used in constructing unsigned transactions
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct UnsignedInput(chain::transaction::UnsignedInput);

#[wasm_bindgen]
impl UnsignedInput {
    /// Create new unsigned input instance from box id and extension
    #[wasm_bindgen(constructor)]
    pub fn new(box_id: &BoxId, ext: &ContextExtension) -> Self {
        UnsignedInput(chain::transaction::UnsignedInput {
            box_id: box_id.clone().into(),
            extension: ext.clone().into(),
        })
    }

    /// Create a new unsigned input from the provided box id
    /// using an empty context extension
    pub fn from_box_id(box_id: &BoxId) -> Self {
        UnsignedInput(chain::transaction::UnsignedInput {
            box_id: box_id.clone().into(),
            extension: ContextExtension::new().into(),
        })
    }

    /// Get box id
    pub fn box_id(&self) -> BoxId {
        self.0.box_id.clone().into()
    }

    /// Get extension
    pub fn extension(&self) -> ContextExtension {
        self.0.extension.clone().into()
    }
}

/// Collection of unsigned signed inputs
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnsignedInputs(Vec<UnsignedInput>);

#[wasm_bindgen]
impl UnsignedInputs {
    /// Create empty UnsignedInputs
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        UnsignedInputs(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> UnsignedInput {
        self.0[index].clone()
    }

    /// Add an element to the collection
    pub fn add(&mut self, b: &UnsignedInput) {
        self.0.push(b.clone());
    }
}

impl From<&UnsignedInputs> for Vec<chain::transaction::UnsignedInput> {
    fn from(v: &UnsignedInputs) -> Self {
        v.0.clone().iter().map(|i| i.0.clone()).collect()
    }
}
impl From<Vec<chain::transaction::UnsignedInput>> for UnsignedInputs {
    fn from(v: Vec<chain::transaction::UnsignedInput>) -> Self {
        UnsignedInputs(v.into_iter().map(UnsignedInput::from).collect())
    }
}

/// Signed inputs used in signed transactions
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct Input(chain::transaction::Input);

#[wasm_bindgen]
impl Input {
    /// Get box id
    pub fn box_id(&self) -> BoxId {
        self.0.box_id.clone().into()
    }

    /// Get the spending proof
    pub fn spending_proof(&self) -> prover_result::ProverResult {
        self.0.spending_proof.clone().into()
    }
}

/// Collection of signed inputs
#[wasm_bindgen]
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Inputs(Vec<Input>);

#[wasm_bindgen]
impl Inputs {
    /// Create empty Inputs
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Inputs(vec![])
    }

    /// Returns the number of elements in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the element of the collection with a given index
    pub fn get(&self, index: usize) -> Input {
        self.0[index].clone()
    }
}

impl From<&Inputs> for Vec<chain::transaction::Input> {
    fn from(v: &Inputs) -> Self {
        v.0.clone().iter().map(|i| i.0.clone()).collect()
    }
}
impl From<Vec<chain::transaction::Input>> for Inputs {
    fn from(v: Vec<chain::transaction::Input>) -> Self {
        Inputs(v.into_iter().map(Input::from).collect())
    }
}

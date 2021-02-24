//! Transaction input
use std::collections::HashMap;
use std::io;

use crate::chain::ergo_box::{BoxId, ErgoBoxId};
use crate::chain::Base16DecodedBytes;
use crate::chain::Base16EncodedBytes;
use ergotree_ir::mir::constant::Constant;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SerializationError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::sigma_protocol::prover::ContextExtension;
use ergotree_ir::sigma_protocol::prover::ProofBytes;
use ergotree_ir::sigma_protocol::prover::ProverResult;
use indexmap::IndexMap;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
extern crate derive_more;
use derive_more::{From, Into};

/// Unsigned (without proofs) transaction input
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct UnsignedInput {
    /// id of the box to spent
    #[cfg_attr(feature = "json", serde(rename = "boxId"))]
    pub box_id: BoxId,
    /// user-defined variables to be put into context
    #[cfg_attr(feature = "json", serde(rename = "extension",))]
    pub extension: WrappedContextExtension,
}

/// IR ContextExtension wrapper (for JSON encoding)
#[cfg_attr(
    feature = "json",
    derive(Serialize, Deserialize),
    serde(
        into = "HashMap<String, Base16EncodedBytes>",
        try_from = "HashMap<String, Base16DecodedBytes>"
    )
)]
#[derive(PartialEq, Debug, Clone, From, Into)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct WrappedContextExtension(ContextExtension);

impl WrappedContextExtension {
    /// Empty context extension
    pub fn empty() -> Self {
        WrappedContextExtension(ContextExtension::empty())
    }

    /// Returns context extension entries
    pub fn values(&self) -> IndexMap<u8, Constant> {
        self.0.values.clone()
    }
}

#[cfg(feature = "json")]
impl Into<HashMap<String, Base16EncodedBytes>> for WrappedContextExtension {
    fn into(self) -> HashMap<String, Base16EncodedBytes> {
        todo!()
    }
}

#[cfg(feature = "json")]
impl std::convert::TryFrom<HashMap<String, Base16DecodedBytes>> for WrappedContextExtension {
    type Error = base16::DecodeError;

    fn try_from(value: HashMap<String, Base16DecodedBytes>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl UnsignedInput {
    /// Create new with empty ContextExtension
    pub fn new(box_id: BoxId, extension: ContextExtension) -> Self {
        todo!()
    }
}

impl<T: ErgoBoxId> From<T> for UnsignedInput {
    fn from(b: T) -> Self {
        UnsignedInput::new(b.box_id(), ContextExtension::empty())
    }
}

/// Fully signed transaction input
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct Input {
    /// id of the box to spent
    #[cfg_attr(feature = "json", serde(rename = "boxId"))]
    pub box_id: BoxId,
    /// proof of spending correctness
    #[cfg_attr(feature = "json", serde(rename = "spendingProof",))]
    pub wrapped_spending_proof: WrappedProverResult,
}

/// Wrapped IR's [`ProverResult`]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
pub struct WrappedProverResult {
    /// Serialized proof bytes
    #[cfg_attr(feature = "json", serde(rename = "proofBytes"))]
    pub proof: Vec<u8>,
    /// Wrapped IR ContextExtension
    #[cfg_attr(feature = "json", serde(rename = "extension"))]
    pub extension: WrappedContextExtension,
}

impl From<ProverResult> for WrappedProverResult {
    fn from(_: ProverResult) -> Self {
        todo!()
    }
}

impl Input {
    /// Create new
    pub fn new(box_id: BoxId, spending_proof: ProverResult) -> Self {
        Self {
            box_id,
            wrapped_spending_proof: spending_proof.into(),
        }
    }

    /// input with an empty proof
    pub fn input_to_sign(&self) -> Input {
        Input {
            box_id: self.box_id.clone(),
            wrapped_spending_proof: ProverResult {
                proof: ProofBytes::Empty,
                extension: self.spending_proof().extension.into(),
            }
            .into(),
        }
    }

    /// Get unwrapped [`ProverResult`]
    pub fn spending_proof(&self) -> ProverResult {
        todo!()
    }
}

impl SigmaSerializable for Input {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.box_id.sigma_serialize(w)?;
        self.spending_proof().sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let box_id = BoxId::sigma_parse(r)?;
        let spending_proof = ProverResult::sigma_parse(r)?;
        Ok(Input {
            box_id,
            wrapped_spending_proof: spending_proof.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ergotree_ir::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Input>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}

//! Transaction input

pub mod prover_result;

use crate::chain::ergo_box::{BoxId, ErgoBoxId};
use crate::chain::json::context_extension::ContextExtensionSerde;
use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_interpreter::sigma_protocol::prover::ProofBytes;
use ergotree_ir::serialization::sigma_byte_reader::SigmaByteRead;
use ergotree_ir::serialization::sigma_byte_writer::SigmaByteWrite;
use ergotree_ir::serialization::SigmaParsingError;
use ergotree_ir::serialization::SigmaSerializable;
use ergotree_ir::serialization::SigmaSerializeResult;
use serde::ser::SerializeStruct;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

use self::prover_result::ProverResult;

/// Unsigned (without proofs) transaction input
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "json", derive(Deserialize))]
pub struct UnsignedInput {
    /// id of the box to spent
    #[cfg_attr(feature = "json", serde(rename = "boxId"))]
    pub box_id: BoxId,
    /// user-defined variables to be put into context
    #[cfg_attr(
        feature = "json",
        serde(rename = "extension",),
        serde(with = "crate::chain::json::context_extension::ContextExtensionSerde")
    )]
    pub extension: ContextExtension,
}

#[cfg(feature = "json")]
impl Serialize for UnsignedInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("UnsignedInput", 2)?;
        s.serialize_field("boxId", &self.box_id)?;
        s.serialize_field(
            "extension",
            &ContextExtensionSerde::from(self.extension.clone()),
        )?;
        s.end()
    }
}

impl UnsignedInput {
    /// Create new with empty ContextExtension
    pub fn new(box_id: BoxId, extension: ContextExtension) -> Self {
        UnsignedInput { box_id, extension }
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
    pub spending_proof: ProverResult,
}

impl Input {
    /// Create new
    pub fn new(box_id: BoxId, spending_proof: ProverResult) -> Self {
        Self {
            box_id,
            spending_proof,
        }
    }

    /// input with an empty proof
    pub fn input_to_sign(&self) -> Input {
        Input {
            box_id: self.box_id.clone(),
            spending_proof: ProverResult {
                proof: ProofBytes::Empty,
                extension: self.spending_proof.extension.clone(),
            },
        }
    }
}

impl SigmaSerializable for Input {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.box_id.sigma_serialize(w)?;
        self.spending_proof.sigma_serialize(w)?;
        Ok(())
    }
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let box_id = BoxId::sigma_parse(r)?;
        let spending_proof = ProverResult::sigma_parse(r)?;
        Ok(Input {
            box_id,
            spending_proof,
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

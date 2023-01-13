use std::str::FromStr;

use crate::chain::json::context_extension::ContextExtensionSerde;
use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use serde::ser::SerializeStruct;
use serde::Serialize;

use super::ProverResult;

impl Serialize for ProverResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ProverResult", 2)?;
        s.serialize_field("proofBytes", &String::from(self.proof.clone()))?;
        s.serialize_field(
            "extension",
            &ContextExtensionSerde::from(self.extension.clone()),
        )?;
        s.end()
    }
}

impl FromStr for ProverResult {
    type Err = base16::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let proof_bytes: Vec<u8> = base16::decode(s)?;
        Ok(ProverResult {
            proof: proof_bytes.into(),
            extension: ContextExtension::empty(),
        })
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use crate::chain::transaction::Input;

    #[test]
    fn parse_proof_explorer_api() {
        // https://github.com/ergoplatform/sigma-rust/issues/670
        let json_str = r#"{
      "id": "61a9e57e6635d02196fadc4ddf40a902c043c692bc6c7b452e02fa467e6e1fd7",
      "value": 6700000,
      "index": 0,
      "spendingProof": "736d882bbfa1767cef64e718eb74f76b96891fb8ad4e8fdd987ec4550b39d7cda0c285f8346e8f842acbae0b3d8e2b52d039cf1dacefc51c",
      "transactionId": "06b02c29a8c1a528c18bab4b6c92d447dc5ff0d99a591ddce2878631c555c97b",
      "outputTransactionId": "88291edf57563b34cb4e4cfae78efb1ad814ef6bff79969b64973a7ec89b59a7",
      "outputIndex": 2,
      "address": "3Wy3BaCjGDWE3bjjZkNo3aWaMz3cYrePMFhchcKovY9uG9vhpAuW"
        }"#;
        let input: Input = serde_json::from_str(json_str).unwrap();
        assert_eq!(input.spending_proof.proof.to_bytes().len(), 56);
    }
}

use super::{
    fiat_shamir::FiatShamirHash,
    unchecked_tree::{UncheckedLeaf, UncheckedSchnorr},
    Challenge, GroupSizedBytes, SigmaBoolean, SigmaProofOfKnowledgeTree, UncheckedSigmaTree,
    UncheckedTree,
};
use k256::Scalar;
use std::convert::{TryFrom, TryInto};

pub fn serialize_sig(tree: UncheckedTree) -> Vec<u8> {
    match tree {
        UncheckedTree::NoProof => vec![],
        UncheckedTree::UncheckedSigmaTree(UncheckedSigmaTree::UncheckedLeaf(
            UncheckedLeaf::UncheckedSchnorr(us),
        )) => {
            let mut res: Vec<u8> = Vec::with_capacity(64);
            res.append(&mut us.challenge.into());
            let mut sm_bytes = us.second_message.z.to_bytes();
            res.append(&mut sm_bytes.as_mut_slice().to_vec());
            res
        }
        _ => todo!(),
    }
}

/**
 * Verifier Step 2: In a top-down traversal of the tree, obtain the challenges for the children of every
 * non-leaf node by reading them from the proof or computing them.
 * Verifier Step 3: For every leaf node, read the response z provided in the proof.
 *
 */
pub fn parse_sig_compute_challenges(
    exp: SigmaBoolean,
    proof_bytes: Vec<u8>,
) -> Result<UncheckedTree, SigParsingError> {
    if proof_bytes.is_empty() {
        Err(SigParsingError::InvalidProofSize)
    } else {
        // Verifier Step 2: Let e_0 be the challenge in the node here (e_0 is called "challenge" in the code)
        let chal_len = super::SOUNDNESS_BYTES;
        let challenge = if let Some(bytes) = proof_bytes.get(..chal_len) {
            // safe since it should only be of the required size
            Challenge::from(FiatShamirHash::try_from(bytes).unwrap())
        } else {
            return Err(SigParsingError::InvalidProofSize);
        };
        match exp {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(dl)) => {
                let scalar_bytes: &[u8; super::GROUP_SIZE] =
                    match proof_bytes.get(chal_len..chal_len + super::GROUP_SIZE) {
                        Some(v) => v.try_into().unwrap(), // safe, since it should only be of this size
                        None => return Err(SigParsingError::InvalidProofSize),
                    };
                let z = Scalar::from(GroupSizedBytes::from(scalar_bytes));
                Ok(UncheckedSchnorr {
                    proposition: dl,
                    commitment_opt: None,
                    challenge,
                    second_message: z.into(),
                }
                .into())
            }
            _ => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SigParsingError {
    InvalidProofSize,
}

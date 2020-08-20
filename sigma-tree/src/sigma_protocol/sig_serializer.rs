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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{Constant, ConstantVal},
        chain::{AddressEncoder, Base16DecodedBytes, NetworkPrefix},
        types::SType,
    };

    #[test]
    fn test_parse_from_mainnet() {
        let spending_proof_input1 = Base16DecodedBytes::try_from("6542a8b8914b103dcbc36d77da3bd58e42ca35755a5190b507764b0bae330b924ce86acfa1b5f9bfc8216c3c4628738e8274d902bea06b48".to_string()).unwrap().0;
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        let decoded_addr = encoder
            .parse_address_from_str("9gmNsqrqdSppLUBqg2UzREmmivgqh1r3jmNcLAc53hk3YCvAGWE")
            .unwrap();
        let ergo_tree = decoded_addr.script().unwrap();
        let sb: SigmaBoolean = match ergo_tree.proposition().unwrap().as_ref() {
            crate::ast::Expr::Const(Constant {
                tpe: SType::SSigmaProp,
                v: ConstantVal::SigmaProp(sp),
            }) => sp.value().clone(),
            _ => panic!(),
        };
        let res = parse_sig_compute_challenges(sb, spending_proof_input1);
        // dbg!(res.clone().unwrap());
        assert!(res.is_ok());
    }
}

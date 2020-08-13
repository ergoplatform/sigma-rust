use super::{SigmaBoolean, UncheckedLeaf, UncheckedSigmaTree, UncheckedTree};

pub fn serialize_sig(tree: UncheckedTree) -> Vec<u8> {
    match tree {
        UncheckedTree::NoProof => vec![],
        UncheckedTree::UncheckedSigmaTree(UncheckedSigmaTree::UncheckedLeaf(
            UncheckedLeaf::UncheckedSchnorr(us),
        )) => {
            let mut res: Vec<u8> = Vec::with_capacity(64);
            res.append(&mut us.challenge.into());
            let mut sm_bytes = us.second_message.0.to_bytes();
            res.append(&mut sm_bytes.as_mut_slice().to_vec());
            res
        }
        _ => todo!(),
    }
}

pub fn parse_sig_compute_challenges(exp: SigmaBoolean, bytes: Vec<u8>) -> UncheckedTree {
    todo!()
}

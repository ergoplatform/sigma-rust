//! Estimation of sigma protocol verification cost from a reduced SigmaBoolean.
//! Constants match Scala sigmastate-interpreter's `estimateCryptoVerifyCost`.

use ergotree_ir::sigma_protocol::sigma_boolean::*;

const PARSE_CHALLENGE: u64 = 10;
const COMPUTE_COMMITMENTS_SCHNORR: u64 = 3400;
const TO_BYTES_SCHNORR: u64 = 570;
const COMPUTE_COMMITMENTS_DHT: u64 = 6450;
const TO_BYTES_DHT: u64 = 680;
const TO_BYTES_CONJUNCTION: u64 = 15;

/// Estimate the cost of verifying a sigma protocol proof for the given proposition.
/// Returns cost in JitCost units (10x block cost scale).
pub fn estimate_crypto_cost(prop: &SigmaBoolean) -> u64 {
    match prop {
        SigmaBoolean::TrivialProp(_) => 0,
        SigmaBoolean::ProofOfKnowledge(pk) => match pk {
            SigmaProofOfKnowledgeTree::ProveDlog(_) => {
                PARSE_CHALLENGE + COMPUTE_COMMITMENTS_SCHNORR + TO_BYTES_SCHNORR
            } // 3980
            SigmaProofOfKnowledgeTree::ProveDhTuple(_) => {
                PARSE_CHALLENGE + COMPUTE_COMMITMENTS_DHT + TO_BYTES_DHT
            } // 7140
        },
        SigmaBoolean::SigmaConjecture(conj) => match conj {
            SigmaConjecture::Cand(cand) => {
                TO_BYTES_CONJUNCTION + cand.items.iter().map(estimate_crypto_cost).sum::<u64>()
            }
            SigmaConjecture::Cor(cor) => {
                TO_BYTES_CONJUNCTION + cor.items.iter().map(estimate_crypto_cost).sum::<u64>()
            }
            SigmaConjecture::Cthreshold(ct) => {
                let n = ct.children.len() as u64;
                let n_coefs = n - ct.k as u64;
                let parse_poly = 10 + 10 * n_coefs;
                let eval_poly = (3 + 3 * n_coefs) * n;
                parse_poly + eval_poly + ct.children.iter().map(estimate_crypto_cost).sum::<u64>()
            }
        },
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use alloc::vec;
    use core::convert::TryInto;
    use ergotree_ir::sigma_protocol::sigma_boolean::cand::Cand;
    use ergotree_ir::sigma_protocol::sigma_boolean::cor::Cor;
    use ergotree_ir::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
    use ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog;
    use sigma_test_util::force_any_val;

    #[test]
    fn test_trivial_prop_cost() {
        assert_eq!(estimate_crypto_cost(&SigmaBoolean::TrivialProp(true)), 0);
        assert_eq!(estimate_crypto_cost(&SigmaBoolean::TrivialProp(false)), 0);
    }

    #[test]
    fn test_prove_dlog_cost() {
        let pd = force_any_val::<ProveDlog>();
        let prop = SigmaBoolean::from(pd);
        assert_eq!(estimate_crypto_cost(&prop), 3980);
    }

    #[test]
    fn test_prove_dh_tuple_cost() {
        use ergotree_ir::sigma_protocol::sigma_boolean::ProveDhTuple;
        let pdh = force_any_val::<ProveDhTuple>();
        let prop = SigmaBoolean::from(pdh);
        assert_eq!(estimate_crypto_cost(&prop), 7140);
    }

    #[test]
    fn test_cand_two_dlog() {
        let pd1 = force_any_val::<ProveDlog>();
        let pd2 = force_any_val::<ProveDlog>();
        let items: SigmaConjectureItems<SigmaBoolean> =
            vec![SigmaBoolean::from(pd1), SigmaBoolean::from(pd2)]
                .try_into()
                .unwrap();
        let cand = Cand { items };
        let prop = SigmaBoolean::from(cand);
        assert_eq!(estimate_crypto_cost(&prop), 15 + 3980 + 3980);
    }

    #[test]
    fn test_cor_two_dlog() {
        let pd1 = force_any_val::<ProveDlog>();
        let pd2 = force_any_val::<ProveDlog>();
        let items: SigmaConjectureItems<SigmaBoolean> =
            vec![SigmaBoolean::from(pd1), SigmaBoolean::from(pd2)]
                .try_into()
                .unwrap();
        let cor = Cor { items };
        let prop = SigmaBoolean::from(cor);
        assert_eq!(estimate_crypto_cost(&prop), 15 + 3980 + 3980);
    }

    #[test]
    fn test_cthreshold_2_of_3_dlog() {
        let pd1 = force_any_val::<ProveDlog>();
        let pd2 = force_any_val::<ProveDlog>();
        let pd3 = force_any_val::<ProveDlog>();
        let children: SigmaConjectureItems<SigmaBoolean> = vec![
            SigmaBoolean::from(pd1),
            SigmaBoolean::from(pd2),
            SigmaBoolean::from(pd3),
        ]
        .try_into()
        .unwrap();
        let ct = Cthreshold { k: 2, children };
        let prop = SigmaBoolean::from(ct);
        // n=3, k=2, n_coefs=1, parse_poly=10+10=20, eval_poly=(3+3)*3=18, children=3*3980=11940
        assert_eq!(estimate_crypto_cost(&prop), 20 + 18 + 11940);
    }
}

use ergotree_interpreter::sigma_protocol::prover::hint::HintsBag;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TransactionHintsBagJson {
    #[serde(rename = "secretHints")]
    secret_hints: Vec<(usize, HintsBag)>,
    #[serde(rename = "publicHints")]
    public_hints: Vec<(usize, HintsBag)>,
}

#[cfg(test)]
mod tests {
    use crate::wallet::multi_sig::TransactionHintsBag;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn roundtrip(t in any::<TransactionHintsBag>()) {
            let j = serde_json::to_string(&t)?;
            eprintln!("{}", j);
            let t_parsed: TransactionHintsBag = serde_json::from_str(&j)?;
            prop_assert_eq![t, t_parsed];
        }

    }
}

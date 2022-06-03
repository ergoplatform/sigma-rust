use std::collections::HashMap;

use ergotree_interpreter::sigma_protocol::prover::hint::{Hint, HintsBag};
use serde::{Deserialize, Serialize};

use crate::wallet::multi_sig::TransactionHintsBag;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct TransactionHintsBagJson {
    #[serde(rename = "secretHints")]
    secret_hints: HashMap<usize, Vec<Hint>>,
    #[serde(rename = "publicHints")]
    public_hints: HashMap<usize, Vec<Hint>>,
}

impl From<TransactionHintsBag> for TransactionHintsBagJson {
    fn from(t: TransactionHintsBag) -> Self {
        let secret_hints = t
            .secret_hints
            .into_iter()
            .map(|(idx, bag)| (idx, bag.hints))
            .collect::<HashMap<usize, Vec<Hint>>>();
        let public_hints = t
            .public_hints
            .into_iter()
            .map(|(idx, bag)| (idx, bag.hints))
            .collect::<HashMap<usize, Vec<Hint>>>();
        TransactionHintsBagJson {
            secret_hints,
            public_hints,
        }
    }
}

impl From<TransactionHintsBagJson> for TransactionHintsBag {
    fn from(tj: TransactionHintsBagJson) -> Self {
        TransactionHintsBag {
            secret_hints: tj
                .secret_hints
                .into_iter()
                .map(|(idx, hints)| (idx, HintsBag { hints }))
                .collect::<HashMap<usize, HintsBag>>(),
            public_hints: tj
                .public_hints
                .into_iter()
                .map(|(idx, hints)| (idx, HintsBag { hints }))
                .collect::<HashMap<usize, HintsBag>>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::multi_sig::TransactionHintsBag;
    use proptest::prelude::*;

    proptest! {

        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn thb_json_roundtrip(t in any::<TransactionHintsBag>()) {
            let json = serde_json::to_string_pretty(&t)?;
            println!("{}", &json);
            let t_parsed: TransactionHintsBag = serde_json::from_str(&json)?;
            prop_assert_eq![t, t_parsed];
        }

    }
}

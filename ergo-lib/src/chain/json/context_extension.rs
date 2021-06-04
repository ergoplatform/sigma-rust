use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_ir::mir::constant::Constant;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg_attr(
    feature = "json",
    derive(Serialize, Deserialize),
    serde(into = "HashMap<String, String>", try_from = "HashMap<String, String>"),
    serde(remote = "ContextExtension")
)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ContextExtensionSerde {
    values: IndexMap<u8, Constant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_context_extension() {
        let mut de = serde_json::Deserializer::from_str("{}");
        let c: ContextExtension = ContextExtensionSerde::deserialize(&mut de).unwrap();
        assert_eq!(c, ContextExtension::empty());
    }

    #[test]
    fn parse_context_extension() {
        let json = r#"
        {"1" :"05b0b5cad8e6dbaef44a", "3":"048ce5d4e505"}
        "#;
        let mut de = serde_json::Deserializer::from_str(json);
        let c: ContextExtension = ContextExtensionSerde::deserialize(&mut de).unwrap();
        assert_eq!(c.values.len(), 2);
        assert!(c.values.get(&1u8).is_some());
        assert!(c.values.get(&3u8).is_some());
    }
}

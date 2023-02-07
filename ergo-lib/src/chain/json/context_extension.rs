use ergotree_interpreter::sigma_protocol::prover::ContextExtension;
use ergotree_ir::{mir::constant::Constant, serialization::SigmaSerializable};
use indexmap::IndexMap;
use serde::{ser::SerializeMap, Deserialize, Serialize};

#[cfg_attr(
    feature = "json",
    derive(Deserialize),
    serde(try_from = "indexmap::IndexMap<String, String>"),
    serde(remote = "ContextExtension")
)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ContextExtensionSerde {
    values: IndexMap<u8, Constant>,
}

#[cfg(feature = "json")]
impl Serialize for ContextExtensionSerde {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        let mut map = serializer.serialize_map(Some(self.values.len()))?;
        for (k, v) in &self.values {
            map.serialize_entry(
                &format!("{}", k),
                &base16::encode_lower(&v.sigma_serialize_bytes().map_err(Error::custom)?),
            )?;
        }
        map.end()
    }
}

impl From<ContextExtension> for ContextExtensionSerde {
    fn from(ce: ContextExtension) -> Self {
        ContextExtensionSerde { values: ce.values }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

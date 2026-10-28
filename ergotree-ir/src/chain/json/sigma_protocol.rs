//! SigmaBoolean JSON encoding

use core::convert::TryFrom;
use core::convert::TryInto;

use super::serialize_bytes;
use crate::serialization::SigmaSerializable;
use crate::sigma_protocol::sigma_boolean::cand::Cand;
use crate::sigma_protocol::sigma_boolean::cor::Cor;
use crate::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
use crate::sigma_protocol::sigma_boolean::ProveDhTuple;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaConjecture;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use bounded_vec::BoundedVecOutOfBounds;
use ergo_chain_types::EcPoint;
use serde::Serialize;
use serde::{Deserialize, Deserializer, Serializer};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "op")]
#[allow(clippy::large_enum_variant)]
pub(crate) enum SigmaBooleanJson {
    #[serde(rename = "205")] // OpCode::PROVE_DLOG
    ProveDlog { h: EcPoint },
    #[serde(rename = "206")] // OpCode::PROVE_DIFFIE_HELLMAN_TUPLE
    ProveDhTuple {
        g: EcPoint,
        h: EcPoint,
        u: EcPoint,
        v: EcPoint,
    },
    #[serde(rename = "300")] // OpCode::TRIVIAL_PROP_FALSE
    TrivialPropFalse { condition: bool },
    #[serde(rename = "301")] // OpCode::TRIVIAL_PROP_TRUE
    TrivialPropTrue { condition: bool },
    #[serde(rename = "150")] // OpCode::AND
    Cand { args: Vec<SigmaBooleanJson> },
    #[serde(rename = "151")] // OpCode::OR
    Cor { args: Vec<SigmaBooleanJson> },
    #[serde(rename = "152")] // OpCode::ATLEAST
    Cthreshold { k: u8, args: Vec<SigmaBooleanJson> },
}

impl From<SigmaBoolean> for SigmaBooleanJson {
    fn from(sb: SigmaBoolean) -> Self {
        match sb {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pd)) => {
                SigmaBooleanJson::ProveDlog { h: *pd.h }
            }
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(pdh)) => {
                SigmaBooleanJson::ProveDhTuple {
                    g: *pdh.g,
                    h: *pdh.h,
                    u: *pdh.u,
                    v: *pdh.v,
                }
            }
            SigmaBoolean::TrivialProp(tp) if tp => {
                SigmaBooleanJson::TrivialPropTrue { condition: tp }
            }
            SigmaBoolean::TrivialProp(tp) => SigmaBooleanJson::TrivialPropFalse { condition: tp },
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(cand)) => SigmaBooleanJson::Cand {
                args: cand
                    .items
                    .as_vec()
                    .clone()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(cor)) => SigmaBooleanJson::Cor {
                args: cor
                    .items
                    .as_vec()
                    .clone()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cthreshold(ct)) => {
                SigmaBooleanJson::Cthreshold {
                    k: ct.k,
                    args: ct
                        .children
                        .as_vec()
                        .clone()
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                }
            }
        }
    }
}

impl TryFrom<SigmaBooleanJson> for SigmaBoolean {
    type Error = BoundedVecOutOfBounds;

    fn try_from(sbj: SigmaBooleanJson) -> Result<Self, Self::Error> {
        Ok(match sbj {
            SigmaBooleanJson::ProveDlog { h } => ProveDlog { h: h.into() }.into(),
            SigmaBooleanJson::ProveDhTuple { g, h, u, v } => ProveDhTuple::new(g, h, u, v).into(),
            SigmaBooleanJson::TrivialPropTrue { condition } => SigmaBoolean::TrivialProp(condition),
            SigmaBooleanJson::TrivialPropFalse { condition } => {
                SigmaBoolean::TrivialProp(condition)
            }
            SigmaBooleanJson::Cand { args } => Cand {
                items: args
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<SigmaBoolean>, _>>()?
                    .try_into()?,
            }
            .into(),
            SigmaBooleanJson::Cor { args } => Cor {
                items: args
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<SigmaBoolean>, _>>()?
                    .try_into()?,
            }
            .into(),
            SigmaBooleanJson::Cthreshold { k, args } => Cthreshold {
                k,
                children: args
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<SigmaBoolean>, _>>()?
                    .try_into()?,
            }
            .into(),
        })
    }
}

/// Serializer (used in Wasm bindings)
pub fn serialize<S>(sb: &SigmaBoolean, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::Error;

    let bytes = sb
        .sigma_serialize_bytes()
        .map_err(|err| Error::custom(err.to_string()))?;
    serialize_bytes(&bytes[..], serializer)
}

/// Deserializer (used in Wasm bindings)
pub fn deserialize<'de, D>(deserializer: D) -> Result<SigmaBoolean, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|str| base16::decode(&str).map_err(|err| Error::custom(err.to_string())))
        .and_then(|bytes| {
            SigmaBoolean::sigma_parse_bytes(&bytes).map_err(|err| Error::custom(err.to_string()))
        })
}

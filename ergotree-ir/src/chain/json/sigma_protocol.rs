use std::convert::TryFrom;
use std::convert::TryInto;

use bounded_vec::BoundedVecOutOfBounds;
use ergo_chain_types::EcPoint;
use serde::Deserialize;
use serde::Serialize;

use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::sigma_protocol::sigma_boolean::cand::Cand;
use crate::sigma_protocol::sigma_boolean::cor::Cor;
use crate::sigma_protocol::sigma_boolean::cthreshold::Cthreshold;
use crate::sigma_protocol::sigma_boolean::ProveDhTuple;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaConjecture;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum SigmaBooleanJson {
    ProveDlog {
        op: OpCode,
        h: EcPoint,
    },
    ProveDhTuple {
        op: OpCode,
        g: EcPoint,
        h: EcPoint,
        u: EcPoint,
        v: EcPoint,
    },
    TrivialProp {
        op: OpCode,
        condition: bool,
    },
    Cand {
        op: OpCode,
        args: Vec<SigmaBooleanJson>,
    },
    Cor {
        op: OpCode,
        args: Vec<SigmaBooleanJson>,
    },
    Cthreshold {
        op: OpCode,
        k: u8,
        args: Vec<SigmaBooleanJson>,
    },
}

impl From<SigmaBoolean> for SigmaBooleanJson {
    fn from(sb: SigmaBoolean) -> Self {
        match sb {
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDlog(pd)) => {
                SigmaBooleanJson::ProveDlog {
                    op: pd.op_code(),
                    h: *pd.h,
                }
            }
            SigmaBoolean::ProofOfKnowledge(SigmaProofOfKnowledgeTree::ProveDhTuple(pdh)) => {
                SigmaBooleanJson::ProveDhTuple {
                    op: pdh.op_code(),
                    g: *pdh.g,
                    h: *pdh.h,
                    u: *pdh.u,
                    v: *pdh.v,
                }
            }
            SigmaBoolean::TrivialProp(tp) => SigmaBooleanJson::TrivialProp {
                op: sb.op_code(),
                condition: tp,
            },
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cand(cand)) => SigmaBooleanJson::Cand {
                op: cand.op_code(),
                args: cand
                    .items
                    .as_vec()
                    .clone()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
            SigmaBoolean::SigmaConjecture(SigmaConjecture::Cor(cor)) => SigmaBooleanJson::Cor {
                op: cor.op_code(),
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
                    op: ct.op_code(),
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
            SigmaBooleanJson::ProveDlog { op: _, h } => ProveDlog { h: h.into() }.into(),
            SigmaBooleanJson::ProveDhTuple { op: _, g, h, u, v } => {
                ProveDhTuple::new(g, h, u, v).into()
            }
            SigmaBooleanJson::TrivialProp { op: _, condition } => {
                SigmaBoolean::TrivialProp(condition)
            }
            SigmaBooleanJson::Cand { op: _, args } => Cand {
                items: args
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<SigmaBoolean>, _>>()?
                    .try_into()?,
            }
            .into(),
            SigmaBooleanJson::Cor { op: _, args } => Cor {
                items: args
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<SigmaBoolean>, _>>()?
                    .try_into()?,
            }
            .into(),
            SigmaBooleanJson::Cthreshold { op: _, k, args } => Cthreshold {
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

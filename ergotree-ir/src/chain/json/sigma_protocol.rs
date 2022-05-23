use std::convert::TryFrom;

use ergo_chain_types::EcPoint;
use serde::Deserialize;
use serde::Serialize;

use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::sigma_protocol::sigma_boolean::ProveDlog;
use crate::sigma_protocol::sigma_boolean::SigmaBoolean;
use crate::sigma_protocol::sigma_boolean::SigmaProofOfKnowledgeTree;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum SigmaBooleanJson {
    ProveDlog { op: OpCode, h: EcPoint },
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
                todo!()
            }
            SigmaBoolean::TrivialProp(_) => todo!(),
            SigmaBoolean::SigmaConjecture(_) => todo!(),
        }
    }
}

impl TryFrom<SigmaBooleanJson> for SigmaBoolean {
    type Error = String;

    fn try_from(sbj: SigmaBooleanJson) -> Result<Self, Self::Error> {
        Ok(match sbj {
            SigmaBooleanJson::ProveDlog { op: _, h } => ProveDlog { h: h.into() }.into(),
        })
    }
}

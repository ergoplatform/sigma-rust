//! Underlying Sigma data types

use crate::{ecpoint::EcPoint, serialization::op_code::OpCode};

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug)]
pub enum SigmaBoolean {
    ProveDHTuple {
        gv: EcPoint,
        hv: EcPoint,
        uv: EcPoint,
        vv: EcPoint,
    },
    ProveDlog(EcPoint),
    CAND(Vec<SigmaBoolean>),
}

impl SigmaBoolean {
    pub fn op_code(&self) -> OpCode {
        match self {
            SigmaBoolean::ProveDHTuple { .. } => todo!(),
            SigmaBoolean::ProveDlog(_) => OpCode::PROVE_DLOG,
            SigmaBoolean::CAND(_) => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct SigmaProp(SigmaBoolean);

impl SigmaProp {
    pub fn new(sbool: SigmaBoolean) -> Self {
        SigmaProp { 0: sbool }
    }

    pub fn value(&self) -> &SigmaBoolean {
        &self.0
    }
}

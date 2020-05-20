//! Underlying Sigma data types

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug)]
pub enum SigmaBoolean {
    ProveDHTuple {
        gv: EcPointType,
        hv: EcPointType,
        uv: EcPointType,
        vv: EcPointType,
    },
    ProveDlog(EcPointType),
    CAND(Vec<SigmaBoolean>),
}

#[derive(PartialEq, Eq, Debug)]
pub struct SigmaProp(SigmaBoolean);

impl SigmaProp {
    pub fn new(sbool: SigmaBoolean) -> Self {
        SigmaProp { 0: sbool }
    }
}

//
#[derive(PartialEq, Eq, Debug)]
pub struct EcPointType {}

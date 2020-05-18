//! Underlying Sigma data types

#[allow(dead_code)]
pub enum SigmaBoolean {
    ProveDlog(u64),
    CAND(Vec<SigmaBoolean>),
}

pub trait SigmaProp {
    fn is_valid(&self) -> bool;
}

pub struct CSigmaProp {
    pub sigma_tree: SigmaBoolean,
}

impl SigmaProp for CSigmaProp {
    fn is_valid(&self) -> bool {
        todo!()
    }
}

pub trait SigmaBox {
    fn value(&self) -> u64;
}
pub struct CSigmaBox {}
impl SigmaBox for CSigmaBox {
    fn value(&self) -> u64 {
        0
    }
}

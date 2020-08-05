//! Constant store for Sigma byte reader

use crate::ast::Constant;

/// Storage for constants used in ErgoTree constant segregation
pub struct ConstantStore {
    constants: Vec<Constant>,
}

pub struct ConstantPlaceholder();

impl ConstantStore {
    /// empty store(no constants)
    pub fn empty() -> Self {
        ConstantStore { constants: vec![] }
    }

    pub fn put(&mut self, c: Constant) -> ConstantPlaceholder {
        self.constants.push(c);
        ConstantPlaceholder()
    }

    pub fn get_all(&self) -> Vec<Constant> {
        self.constants.clone()
    }
}


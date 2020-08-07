//! Constant store for Sigma byte reader

use crate::ast::{Constant, ConstantPlaceholder};

/// Storage for constants used in ErgoTree constant segregation
pub struct ConstantStore {
    constants: Vec<Constant>,
}

impl ConstantStore {
    pub fn empty() -> Self {
        ConstantStore { constants: vec![] }
    }

    pub fn new(constants: Vec<Constant>) -> Self {
        ConstantStore { constants }
    }

    pub fn get(&self, index: u32) -> Option<&Constant> {
        self.constants.get(index as usize)
    }

    pub fn put(&mut self, c: Constant) -> ConstantPlaceholder {
        self.constants.push(c.clone());
        assert!(self.constants.len() <= u32::MAX as usize);
        ConstantPlaceholder {
            id: (self.constants.len() - 1) as u32,
            tpe: c.tpe,
        }
    }

    pub fn get_all(&self) -> Vec<Constant> {
        self.constants.clone()
    }
}

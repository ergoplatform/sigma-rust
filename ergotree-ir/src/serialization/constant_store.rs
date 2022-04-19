//! Constant store for Sigma byte reader

use crate::mir::constant::{Constant, ConstantPlaceholder};

/// Storage for constants used in ErgoTree constant segregation
pub struct ConstantStore {
    constants: Vec<Constant>,
}

impl ConstantStore {
    /// Make an empty(no constants) store
    pub fn empty() -> Self {
        ConstantStore { constants: vec![] }
    }

    /// Make a store with constants provided
    pub fn new(constants: Vec<Constant>) -> Self {
        ConstantStore { constants }
    }

    /// Get a constant by it's index in serialized constants list (stored in ConstantPlaceholder)
    pub fn get(&self, index: u32) -> Option<&Constant> {
        self.constants.get(index as usize)
    }

    /// Save a Constant and get ConstantPlaceholder(with stored index) back
    pub fn put(&mut self, c: Constant) -> ConstantPlaceholder {
        self.constants.push(c.clone());
        assert!(self.constants.len() <= u32::MAX as usize);
        ConstantPlaceholder {
            id: (self.constants.len() - 1) as u32,
            tpe: c.tpe,
        }
    }

    /// Get all stored constants (order preserved)
    pub fn get_all(&self) -> Vec<Constant> {
        self.constants.clone()
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::constant::Literal;
    use crate::types::stype::SType;

    #[test]
    fn test_empty() {
        let s = ConstantStore::empty();
        assert!(s.get_all().is_empty());
        assert!(s.get(0).is_none());
    }

    #[test]
    fn test_non_empty() {
        let c = Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(true),
        };
        let s = ConstantStore::new(vec![c.clone()]);
        assert!(s.get(0).is_some());
        assert_eq!(s.get(0).unwrap().clone(), c);
        assert!(!s.get_all().is_empty());
        assert_eq!(s.get_all().get(0).unwrap().clone(), c);
    }

    #[test]
    fn test_put() {
        let c = Constant {
            tpe: SType::SBoolean,
            v: Literal::Boolean(true),
        };
        let mut s = ConstantStore::empty();
        s.put(c.clone());
        assert!(s.get(0).is_some());
        assert_eq!(s.get(0).unwrap().clone(), c);
        assert!(!s.get_all().is_empty());
        assert_eq!(s.get_all().get(0).unwrap().clone(), c);
    }
}

use std::collections::HashMap;

use crate::mir::val_def::ValId;
use crate::types::stype::SType;

pub struct ValDefTypeStore(HashMap<ValId, SType>);

impl ValDefTypeStore {
    pub fn new() -> Self {
        ValDefTypeStore(HashMap::new())
    }

    pub fn insert(&mut self, id: ValId, tpe: SType) {
        self.0.insert(id, tpe);
    }

    pub fn get(&self, id: &ValId) -> Option<&SType> {
        self.0.get(id)
    }
}

impl Default for ValDefTypeStore {
    fn default() -> Self {
        ValDefTypeStore::new()
    }
}

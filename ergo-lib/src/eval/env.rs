use std::collections::HashMap;

use crate::ast::value::Value;

/// Environment for the interpreter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Env {
    store: HashMap<i32, Value>,
}

impl Env {
    /// Empty environment
    pub fn empty() -> Env {
        Env {
            store: HashMap::new(),
        }
    }

    /// Extend this environment (create new) with added element
    pub fn extend(&self, idx: i32, v: Value) -> Env {
        let mut new_store = self.store.clone();
        new_store.insert(idx, v);
        Env { store: new_store }
    }

    /// Get an element
    pub fn get(&self, idx: i32) -> Option<&Value> {
        self.store.get(&idx)
    }
}

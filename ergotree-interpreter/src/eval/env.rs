use std::collections::HashMap;
use std::fmt::Display;

use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::value::Value;

/// Environment for the interpreter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Env {
    store: HashMap<ValId, Value>,
}

impl Env {
    /// Empty environment
    pub fn empty() -> Env {
        Env {
            store: HashMap::new(),
        }
    }

    /// Returns `true` if the environment is empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Extend this environment (create new) with added element
    pub fn extend(&self, idx: ValId, v: Value) -> Env {
        let mut new_store = self.store.clone();
        new_store.insert(idx, v);
        Env { store: new_store }
    }

    /// Insert a Value for the given ValId
    pub fn insert(&mut self, idx: ValId, v: Value) {
        self.store.insert(idx, v);
    }

    /// Remove a Value for the given ValId
    pub fn remove(&mut self, idx: &ValId) {
        self.store.remove(idx);
    }

    /// Get an element
    pub fn get(&self, idx: ValId) -> Option<&Value> {
        self.store.get(&idx)
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<&ValId> = self.store.keys().collect();
        keys.sort();
        for k in keys {
            writeln!(f, "v{}: {}", k, self.store[k])?;
        }
        Ok(())
    }
}

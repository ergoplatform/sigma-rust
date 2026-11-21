use core::fmt::Display;
use hashbrown::HashMap;

use alloc::vec::Vec;
use ergotree_ir::mir::val_def::ValId;
use ergotree_ir::mir::value::Value;

/// Environment for the interpreter
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Env<'ctx> {
    store: HashMap<ValId, Value<'ctx>>,
}

impl<'ctx> Env<'ctx> {
    /// Empty environment
    pub fn empty() -> Env<'ctx> {
        Env {
            store: HashMap::new(),
        }
    }

    /// Returns `true` if the environment is empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Extend this environment (create new) with added element
    pub fn extend(&self, idx: ValId, v: Value<'ctx>) -> Env<'ctx> {
        let mut new_store = self.store.clone();
        new_store.insert(idx, v);
        Env { store: new_store }
    }

    /// Insert a Value for the given ValId
    pub fn insert(&mut self, idx: ValId, v: Value<'ctx>) {
        self.store.insert(idx, v);
    }

    /// Remove a Value for the given ValId
    pub fn remove(&mut self, idx: &ValId) {
        self.store.remove(idx);
    }

    /// Get an element
    pub fn get(&self, idx: ValId) -> Option<&Value<'ctx>> {
        self.store.get(&idx)
    }

    /// Snapshot the current bindings — used to capture the defining
    /// environment when a `FuncValue` evaluates to a lambda value.
    pub(crate) fn bindings(&self) -> Vec<(ValId, Value<'ctx>)> {
        self.store.iter().map(|(&k, v)| (k, v.clone())).collect()
    }
    /// Convert borrowed data to Arc
    pub(crate) fn to_static(&'ctx self) -> Env<'static> {
        Env {
            store: self
                .store
                .iter()
                .map(|(&k, v)| (k, v.to_static()))
                .collect(),
        }
    }
}

impl Display for Env<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut keys: Vec<&ValId> = self.store.keys().collect();
        keys.sort();
        for k in keys {
            writeln!(f, "v{}: {}", k, self.store[k])?;
        }
        Ok(())
    }
}

//! Compilation environment

use std::collections::HashMap;

use ergotree_ir::mir::constant::Constant;

/// Environment with values substituted for identifiers during compilation
pub struct ScriptEnv(HashMap<String, Constant>);

impl Default for ScriptEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEnv {
    /// Empty environment
    pub fn new() -> Self {
        ScriptEnv(HashMap::new())
    }

    /// Returns the value([`Constant`]) for the given identifier (if any)
    pub fn get(&self, ident: &str) -> Option<&Constant> {
        self.0.get(ident)
    }
}

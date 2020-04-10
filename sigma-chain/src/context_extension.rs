use std::collections::HashMap;

pub struct ContextExtension {
    pub values: HashMap<u8, Box<[u8]>>,
}

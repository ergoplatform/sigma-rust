use std::fmt::Debug;

use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeId(pub u8);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeCompanionHead {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeCompanion {
    head: &'static STypeCompanionHead,
    methods: Vec<&'static SMethodDesc>,
}

impl STypeCompanion {
    pub fn new(head: &'static STypeCompanionHead, methods: Vec<&'static SMethodDesc>) -> Self {
        STypeCompanion { head, methods }
    }

    pub fn method_by_id(&'static self, method_id: MethodId) -> Option<SMethod> {
        self.methods
            .iter()
            .find(|m| m.method_id == method_id)
            .map(|m| m.as_method(self))
    }

    pub fn methods(&'static self) -> Vec<SMethod> {
        self.methods.iter().map(|m| m.as_method(self)).collect()
    }
}

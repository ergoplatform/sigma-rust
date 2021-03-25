use std::fmt::Debug;

use crate::serialization::types::TypeCode;

use super::sbox;
use super::scoll;
use super::scontext;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct STypeCompanionHead {
    pub type_id: TypeCode,
    pub type_name: &'static str,
}

/// Object's type companion
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeCompanion {
    head: &'static STypeCompanionHead,
    methods: Vec<&'static SMethodDesc>,
}

impl STypeCompanion {
    pub(crate) const fn new(
        head: &'static STypeCompanionHead,
        methods: Vec<&'static SMethodDesc>,
    ) -> Self {
        STypeCompanion { head, methods }
    }

    /// Get type companion for a givec type id
    pub fn type_by_id(type_id: TypeCode) -> &'static STypeCompanion {
        if type_id == scontext::S_CONTEXT_TYPE_COMPANION.type_id() {
            &scontext::S_CONTEXT_TYPE_COMPANION
        } else if type_id == sbox::S_BOX_TYPE_COMPANION.type_id() {
            &sbox::S_BOX_TYPE_COMPANION
        } else if type_id == scoll::S_COLL_TYPE_COMPANION.type_id() {
            &scoll::S_COLL_TYPE_COMPANION
        } else {
            todo!("cannot find STypeCompanion for {0:?} type id", type_id)
        }
    }

    /// Get method signature for this object by a method id
    pub fn method_by_id(&'static self, method_id: &MethodId) -> Option<SMethod> {
        self.methods
            .iter()
            .find(|m| m.method_id == *method_id)
            .map(|m| m.as_method(self))
    }

    /// Get list of method signatures for this object's type companion
    pub fn methods(&'static self) -> Vec<SMethod> {
        self.methods.iter().map(|m| m.as_method(self)).collect()
    }

    /// Get object type id for this type companion
    pub fn type_id(&'static self) -> TypeCode {
        self.head.type_id
    }

    /// Get object's type name
    pub fn type_name(&'static self) -> &'static str {
        self.head.type_name
    }
}

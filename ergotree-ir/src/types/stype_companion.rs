use std::convert::TryFrom;
use std::fmt::Debug;

use crate::serialization::types::TypeCode;
use crate::serialization::SigmaParsingError;

use super::sbox;
use super::scoll;
use super::scontext;
use super::sglobal;
use super::sgroup_elem;
use super::sheader;
use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::soption;
use super::spreheader;

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct STypeCompanionHead {
    pub type_id: TypeCode,
    pub type_name: &'static str,
}

/// Object's type companion
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum STypeCompanion {
    /// Context
    Context,
    /// Box
    Box,
    /// Coll
    Coll,
    /// Group element
    GroupElem,
    /// Global
    Global,
    /// Header
    Header,
    /// Pre-header
    PreHeader,
    /// Option
    Option,
}

impl STypeCompanion {
    fn s_type_companion_head(&self) -> &STypeCompanionHead {
        match self {
            STypeCompanion::Context => &scontext::TYPE_COMPANION_HEAD,
            STypeCompanion::Box => &sbox::TYPE_COMPANION_HEAD,
            STypeCompanion::Coll => &scoll::TYPE_COMPANION_HEAD,
            STypeCompanion::GroupElem => &sgroup_elem::TYPE_COMPANION_HEAD,
            STypeCompanion::Global => &sglobal::TYPE_COMPANION_HEAD,
            STypeCompanion::Header => &sheader::TYPE_COMPANION_HEAD,
            STypeCompanion::PreHeader => &spreheader::TYPE_COMPANION_HEAD,
            STypeCompanion::Option => &soption::TYPE_COMPANION_HEAD,
        }
    }

    fn method_desc<'a>(&'a self) -> &'a Vec<&'static SMethodDesc> {
        match self {
            STypeCompanion::Context => &*scontext::METHOD_DESC,
            STypeCompanion::Box => &*sbox::METHOD_DESC,
            STypeCompanion::Coll => &*scoll::METHOD_DESC,
            STypeCompanion::GroupElem => &*sgroup_elem::METHOD_DESC,
            STypeCompanion::Global => &*sglobal::METHOD_DESC,
            STypeCompanion::Header => &*sheader::METHOD_DESC,
            STypeCompanion::PreHeader => &*spreheader::METHOD_DESC,
            STypeCompanion::Option => &*soption::METHOD_DESC,
        }
    }

    /// Get method signature for this object by a method id
    pub fn method_by_id(&self, method_id: &MethodId) -> Option<SMethod> {
        self.method_desc()
            .iter()
            .find(|m| m.method_id == *method_id)
            .map(|m| m.as_method(*self))
    }

    /// Get list of method signatures for this object's type companion
    pub fn methods(&self) -> Vec<SMethod> {
        self.method_desc()
            .iter()
            .map(|m| m.as_method(*self))
            .collect()
    }

    /// Get object type id for this type companion
    pub fn type_id(&self) -> TypeCode {
        self.s_type_companion_head().type_id
    }

    /// Get object's type name
    pub fn type_name(&self) -> &'static str {
        self.s_type_companion_head().type_name
    }
}

impl TryFrom<TypeCode> for STypeCompanion {
    type Error = SigmaParsingError;
    fn try_from(value: TypeCode) -> Result<Self, Self::Error> {
        for (type_code, type_companion) in [
            STypeCompanion::Context,
            STypeCompanion::Box,
            STypeCompanion::Coll,
            STypeCompanion::GroupElem,
            STypeCompanion::Global,
            STypeCompanion::Header,
            STypeCompanion::PreHeader,
            STypeCompanion::Option,
        ]
        .iter()
        .map(|v| (v.type_id(), *v))
        {
            if type_code == value {
                return Ok(type_companion);
            }
        }
        Err(SigmaParsingError::NotImplementedYet(format!(
            "cannot find STypeCompanion for {0:?} type id",
            value,
        )))
    }
}

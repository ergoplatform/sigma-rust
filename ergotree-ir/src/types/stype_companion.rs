use std::convert::TryFrom;
use std::fmt::Debug;

use crate::serialization::types::TypeCode;
use crate::serialization::SigmaParsingError;

use super::savltree;
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
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Object's type companion
#[derive(PartialEq, Eq, Debug, Clone, Copy, EnumIter)]
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
    /// AVL tree
    AvlTree,
}

impl STypeCompanion {
    fn method_desc<'a>(&'a self) -> &'a Vec<&'static SMethodDesc> {
        match self {
            STypeCompanion::Context => &scontext::METHOD_DESC,
            STypeCompanion::Box => &sbox::METHOD_DESC,
            STypeCompanion::Coll => &scoll::METHOD_DESC,
            STypeCompanion::GroupElem => &sgroup_elem::METHOD_DESC,
            STypeCompanion::Global => &sglobal::METHOD_DESC,
            STypeCompanion::Header => &sheader::METHOD_DESC,
            STypeCompanion::PreHeader => &spreheader::METHOD_DESC,
            STypeCompanion::Option => &soption::METHOD_DESC,
            STypeCompanion::AvlTree => &savltree::METHOD_DESC,
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

    /// Get object's type code
    pub fn type_code(&self) -> TypeCode {
        match self {
            STypeCompanion::Context => scontext::TYPE_CODE,
            STypeCompanion::Box => sbox::TYPE_CODE,
            STypeCompanion::Coll => scoll::TYPE_CODE,
            STypeCompanion::GroupElem => sgroup_elem::TYPE_CODE,
            STypeCompanion::Global => sglobal::TYPE_CODE,
            STypeCompanion::Header => sheader::TYPE_CODE,
            STypeCompanion::PreHeader => spreheader::TYPE_CODE,
            STypeCompanion::Option => soption::TYPE_CODE,
            STypeCompanion::AvlTree => savltree::TYPE_CODE,
        }
    }

    /// Get object's type name
    pub fn type_name(&self) -> &'static str {
        match self {
            STypeCompanion::Context => scontext::TYPE_NAME,
            STypeCompanion::Box => sbox::TYPE_NAME,
            STypeCompanion::Coll => scoll::TYPE_NAME,
            STypeCompanion::GroupElem => sgroup_elem::TYPE_NAME,
            STypeCompanion::Global => sglobal::TYPE_NAME,
            STypeCompanion::Header => sheader::TYPE_NAME,
            STypeCompanion::PreHeader => spreheader::TYPE_NAME,
            STypeCompanion::Option => soption::TYPE_NAME,
            STypeCompanion::AvlTree => savltree::TYPE_NAME,
        }
    }
}

impl TryFrom<TypeCode> for STypeCompanion {
    type Error = SigmaParsingError;
    fn try_from(value: TypeCode) -> Result<Self, Self::Error> {
        for (type_code, type_companion) in STypeCompanion::iter().map(|v| (v.type_code(), v)) {
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

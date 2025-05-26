use core::convert::TryFrom;
use core::fmt::Debug;

use crate::serialization::types::TypeCode;
use crate::soft_fork::SoftForkError;

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
use super::snumeric::sbigint;
use super::snumeric::sbyte;
use super::snumeric::sint;
use super::snumeric::slong;
use super::snumeric::sshort;
use super::snumeric::sunsignedbigint;
use super::soption;
use super::spreheader;

use alloc::vec::Vec;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Object's type companion
#[derive(PartialEq, Eq, Debug, Clone, Copy, EnumIter)]
pub enum STypeCompanion {
    /// Signed Byte
    SByte,
    /// 16-bit signed integer value
    SShort,
    /// 32-bit signed integer value
    SInt,
    /// 64-bit signed integer value
    SLong,
    /// 256-bit signed integer value
    SBigInt,
    /// 256-bit unsigned integer value
    SUnsignedBigInt,
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
    fn method_desc(&self) -> &[SMethodDesc] {
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
            STypeCompanion::SByte => &sbyte::METHOD_DESC,
            STypeCompanion::SShort => &sshort::METHOD_DESC,
            STypeCompanion::SInt => &sint::METHOD_DESC,
            STypeCompanion::SLong => &slong::METHOD_DESC,
            STypeCompanion::SBigInt => &sbigint::METHOD_DESC,
            STypeCompanion::SUnsignedBigInt => &sunsignedbigint::METHOD_DESC,
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
            STypeCompanion::SByte => sbyte::TYPE_CODE,
            STypeCompanion::SShort => sshort::TYPE_CODE,
            STypeCompanion::SInt => sint::TYPE_CODE,
            STypeCompanion::SLong => slong::TYPE_CODE,
            STypeCompanion::SBigInt => sbigint::TYPE_CODE,
            STypeCompanion::SUnsignedBigInt => sunsignedbigint::TYPE_CODE,
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
            STypeCompanion::SByte => sbyte::TYPE_NAME,
            STypeCompanion::SShort => sshort::TYPE_NAME,
            STypeCompanion::SInt => sint::TYPE_NAME,
            STypeCompanion::SLong => slong::TYPE_NAME,
            STypeCompanion::SBigInt => sbigint::TYPE_NAME,
            STypeCompanion::SUnsignedBigInt => sunsignedbigint::TYPE_NAME,
        }
    }
}

impl TryFrom<TypeCode> for STypeCompanion {
    type Error = SoftForkError;
    fn try_from(value: TypeCode) -> Result<Self, Self::Error> {
        for (type_code, type_companion) in STypeCompanion::iter().map(|v| (v.type_code(), v)) {
            if type_code == value {
                return Ok(type_companion);
            }
        }
        Err(SoftForkError::NoMethods(value))
    }
}

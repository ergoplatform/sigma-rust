use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::types::TypeCode;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use std::io::Error;

use super::stype::SType;
use super::stype_companion::STypeCompanion;

/// Method id unique among the methods of the same object
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodId(pub u8);

impl SigmaSerializable for MethodId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        w.put_u8(self.0)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        Ok(Self(r.get_u8()?))
    }
}

/// Object method signature
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SMethod {
    /// Object type companion
    pub obj_type: &'static STypeCompanion,
    method_raw: &'static SMethodDesc,
}

impl SMethod {
    /// Create new SMethod
    pub(crate) fn new(
        obj_type: &'static STypeCompanion,
        method_raw: &'static SMethodDesc,
    ) -> SMethod {
        SMethod {
            obj_type,
            method_raw,
        }
    }

    /// Get method from type and method ids
    pub fn from_ids(type_id: TypeCode, method_id: MethodId) -> Self {
        let obj_type = STypeCompanion::type_by_id(type_id);
        match obj_type.method_by_id(&method_id) {
            Some(m) => m,
            None => panic!(
                "no method id {0:?} found in type companion with type id {1:?}",
                method_id, type_id
            ),
        }
    }

    /// Type
    pub fn tpe(&self) -> &SType {
        &self.method_raw.tpe
    }

    /// Returns method name
    pub fn name(&self) -> &'static str {
        self.method_raw.name
    }

    /// Returns method id
    pub fn method_id(&self) -> MethodId {
        self.method_raw.method_id.clone()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct SMethodDesc {
    pub(crate) name: &'static str,
    pub(crate) method_id: MethodId,
    pub(crate) tpe: SType,
}

impl SMethodDesc {
    pub(crate) fn as_method(&'static self, obj_type: &'static STypeCompanion) -> SMethod {
        SMethod {
            obj_type,
            method_raw: self,
        }
    }
}

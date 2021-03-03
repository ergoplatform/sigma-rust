use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use std::io::Error;

use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_companion::TypeId;

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

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SMethod {
    pub obj_type: &'static STypeCompanion,
    method_raw: &'static SMethodDesc,
}

impl SMethod {
    pub fn new(obj_type: &'static STypeCompanion, method_raw: &'static SMethodDesc) -> SMethod {
        SMethod {
            obj_type,
            method_raw,
        }
    }

    pub fn from_ids(type_id: TypeId, method_id: MethodId) -> Self {
        let obj_type = STypeCompanion::type_by_id(type_id);
        match obj_type.method_by_id(&method_id) {
            Some(m) => m,
            None => panic!(
                "no method id {0:?} found in type companion with type id {1:?}",
                method_id, type_id
            ),
        }
    }

    pub fn tpe(&self) -> &SType {
        &self.method_raw.tpe
    }

    pub fn name(&self) -> &'static str {
        self.method_raw.name
    }

    pub fn method_id(&self) -> MethodId {
        self.method_raw.method_id.clone()
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SMethodDesc {
    pub name: &'static str,
    pub method_id: MethodId,
    pub tpe: SType,
}

impl SMethodDesc {
    pub fn as_method(&'static self, obj_type: &'static STypeCompanion) -> SMethod {
        SMethod {
            obj_type,
            method_raw: self,
        }
    }
}

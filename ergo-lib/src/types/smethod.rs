use super::stype::SType;
use super::stype_companion::STypeCompanion;

/// Method id unique among the methods of the same object
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodId(pub u8);

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

use super::stype::SType;
use super::stype_companion::STypeCompanion;

/// Method id unique among the methods of the same object
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodId(u8);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SMethod {
    pub obj_type: Box<STypeCompanion>,
    pub name: String,
    pub method_id: MethodId,
    pub tpe: SType,
}

use super::smethod::SMethod;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TypeId(u8);

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STypeCompanion {
    pub type_id: TypeId,
    pub type_name: String,
    pub methods: Vec<SMethod>,
}

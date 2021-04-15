#![allow(missing_docs)]

use crate::serialization::types::TypeCode;

use super::smethod::MethodId;
use super::smethod::SMethod;
use super::smethod::SMethodDesc;
use super::stype::SType;
use super::stype::SType::{SColl,SBox};
use super::stype_companion::STypeCompanion;
use super::stype_companion::STypeCompanionHead;
use lazy_static::lazy_static;

pub const TYPE_ID: TypeCode = TypeCode::SCONTEXT;

static S_CONTEXT_TYPE_COMPANION_HEAD: STypeCompanionHead = STypeCompanionHead {
    type_id: TYPE_ID,
    type_name: "Context",
};

pub const DATA_INPUTS_PROPERTY_METHOD_ID: MethodId = MethodId(1);
lazy_static! {
    static ref DATA_INPUTS_PROPERTY_METHOD_DESC: SMethodDesc =
    property("dataInputs", SColl(SBox.into()).into(), DATA_INPUTS_PROPERTY_METHOD_ID);
}

lazy_static! {
    pub static ref S_CONTEXT_TYPE_COMPANION: STypeCompanion = STypeCompanion::new(
        &S_CONTEXT_TYPE_COMPANION_HEAD,
        vec![&DATA_INPUTS_PROPERTY_METHOD_DESC]
    );
}

lazy_static! {
    pub static ref DATA_INPUTS_PROPERTY: SMethod = SMethod::new(
        &S_CONTEXT_TYPE_COMPANION,
        DATA_INPUTS_PROPERTY_METHOD_DESC.clone(),
    );
}

fn property(name: &'static str, res_tpe: SType, id: MethodId) -> SMethodDesc {
    SMethodDesc::property(SType::SContext, name, res_tpe, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ids() {
        assert!(
            SMethod::from_ids(TYPE_ID, DATA_INPUTS_PROPERTY_METHOD_ID).map(|e| e.name())
                == Ok("dataInputs")
        );
    }
}

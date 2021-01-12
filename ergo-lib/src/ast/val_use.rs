use crate::types::stype::SType;

/** Special node which represents a reference to ValDef in was introduced as result of CSE. */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValUse {
    pub val_id: i32,
    pub tpe: SType,
}

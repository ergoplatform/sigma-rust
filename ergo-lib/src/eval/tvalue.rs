use crate::ast::value::Value;
use crate::types::stype::LiftIntoSType;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TValue<T: LiftIntoSType> {
    pub v: T,
}

impl<T: LiftIntoSType> From<T> for TValue<T> {
    fn from(raw: T) -> Self {
        TValue { v: raw }
    }
}

impl<T: LiftIntoSType + Into<Value>> From<TValue<T>> for Value {
    fn from(tv: TValue<T>) -> Self {
        // Value::from(tv.v)
        tv.v.into()
    }
}

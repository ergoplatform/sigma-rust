use std::io;

use crate::eval::env::Env;
use crate::eval::EvalContext;
use crate::eval::EvalError;
use crate::eval::Evaluable;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::val_def::ValId;
use super::value::Value;

/** Special node which represents a reference to ValDef in was introduced as result of CSE. */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValUse {
    pub val_id: ValId,
    pub tpe: SType,
}

impl ValUse {
    pub fn op_code(&self) -> OpCode {
        OpCode::VAL_USE
    }
}

impl Evaluable for ValUse {
    fn eval(&self, env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        env.get(self.val_id).cloned().ok_or_else(|| {
            EvalError::NotFound(format!("no value in env for id: {0:?}", self.val_id))
        })
    }
}

impl SigmaSerializable for ValUse {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.val_id.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let val_id = ValId::sigma_parse(r)?;
        let tpe = r
            .val_def_type_store()
            .get(&val_id)
            .ok_or(SerializationError::ValDefIdNotFound(val_id))?
            .clone();
        Ok(ValUse { val_id, tpe })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::block::BlockValue;
    use crate::ast::expr::Expr;
    use crate::ast::func_value::FuncArg;
    use crate::ast::func_value::FuncValue;
    use crate::ast::val_def::ValDef;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip_block_value(v in any::<ValDef>()) {
            // ValDef should put the type into the ValDefStore for ValUse to read
            let block: Expr = BlockValue {
                items: vec![v.clone()],
                result: Box::new(ValUse{ val_id: v.id, tpe: v.tpe() }.into()),
            }.into();
            prop_assert_eq![sigma_serialize_roundtrip(&block), block];
        }

        #[test]
        fn ser_roundtrip_func_value(v in any::<FuncArg>()) {
            let body = ValUse{ val_id: v.idx, tpe: v.tpe.clone() }.into();
            let func: Expr = FuncValue::new(vec![v], body).into();
            prop_assert_eq![sigma_serialize_roundtrip(&func), func];
        }

    }
}

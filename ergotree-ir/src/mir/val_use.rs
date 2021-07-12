use std::io;

use super::val_def::ValId;
use crate::has_opcode::HasStaticOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

/** Special node which represents a reference to ValDef in was introduced as result of CSE. */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ValUse {
    /// Variable id
    pub val_id: ValId,
    /// Variable type
    pub tpe: SType,
}

impl HasStaticOpCode for ValUse {
    const OP_CODE: OpCode = OpCode::VAL_USE;
}

impl SigmaSerializable for ValUse {
    fn sigma_serialize<W: SigmaByteWrite>(
        &self,
        w: &mut W,
    ) -> crate::serialization::SigmaSerializeResult {
        self.val_id.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let val_id = ValId::sigma_parse(r)?;
        let tpe = r
            .val_def_type_store()
            .get(&val_id)
            .ok_or(SigmaParsingError::ValDefIdNotFound(val_id))?
            .clone();
        Ok(ValUse { val_id, tpe })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use crate::mir::block::BlockValue;
    use crate::mir::expr::Expr;
    use crate::mir::func_value::FuncArg;
    use crate::mir::func_value::FuncValue;
    use crate::mir::val_def::ValDef;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip_block_value(v in any::<ValDef>()) {
            // ValDef should put the type into the ValDefStore for ValUse to read
            let block: Expr = BlockValue {
                items: vec![v.clone().into()],
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

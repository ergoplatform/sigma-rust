use super::bin_op::bin_op_sigma_parse;
use super::bin_op::bin_op_sigma_serialize;
use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::ast::bin_op::LogicOp;
use crate::ast::block::BlockValue;
use crate::ast::coll_fold::Fold;
use crate::ast::constant::Constant;
use crate::ast::constant::ConstantPlaceholder;
use crate::ast::expr::Expr;
use crate::ast::extract_amount::ExtractAmount;
use crate::ast::extract_reg_as::ExtractRegisterAs;
use crate::ast::func_value::FuncValue;
use crate::ast::global_vars::GlobalVars;
use crate::ast::method_call::MethodCall;
use crate::ast::option_get::OptionGet;
use crate::ast::property_call::PropertyCall;
use crate::ast::select_field::SelectField;
use crate::ast::val_def::ValDef;
use crate::ast::val_use::ValUse;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};

use std::io;

impl SigmaSerializable for Expr {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        match self {
            Expr::Const(c) => match w.constant_store() {
                Some(cs) => {
                    let ph = cs.put(*c.clone());
                    ph.op_code().sigma_serialize(w)?;
                    ph.sigma_serialize(w)
                }
                None => c.sigma_serialize(w),
            },
            expr => {
                let op_code = self.op_code();
                op_code.sigma_serialize(w)?;
                match expr {
                    Expr::Fold(op) => op.sigma_serialize(w),
                    Expr::ConstPlaceholder(cp) => cp.sigma_serialize(w),
                    Expr::GlobalVars(_) => Ok(()),
                    Expr::MethodCall(mc) => mc.sigma_serialize(w),
                    Expr::ProperyCall(pc) => pc.sigma_serialize(w),
                    Expr::Context => Ok(()),
                    Expr::OptionGet(v) => v.sigma_serialize(w),
                    Expr::ExtractRegisterAs(v) => v.sigma_serialize(w),
                    Expr::BinOp(op) => bin_op_sigma_serialize(op, w),
                    Expr::BlockValue(op) => op.sigma_serialize(w),
                    Expr::ValUse(op) => op.sigma_serialize(w),
                    Expr::ValDef(op) => op.sigma_serialize(w),
                    Expr::FuncValue(op) => op.sigma_serialize(w),
                    Expr::ExtractAmount(op) => op.sigma_serialize(w),
                    Expr::SelectField(op) => op.sigma_serialize(w),
                    _ => panic!(format!("don't know how to serialize {:?}", expr)),
                }
            }
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let first_byte = match r.peek_u8() {
            Ok(b) => Ok(b),
            Err(_) => {
                let res = r.get_u8(); // get(consume) the error
                assert!(res.is_err());
                res
            }
        }?;
        if first_byte <= OpCode::LAST_CONSTANT_CODE.value() {
            let constant = Constant::sigma_parse(r)?;
            Ok(Expr::Const(constant.into()))
        } else {
            let op_code = OpCode::sigma_parse(r)?;
            match op_code {
                OpCode::FOLD => Ok(Box::new(Fold::sigma_parse(r)?).into()),
                ConstantPlaceholder::OP_CODE => {
                    let cp = ConstantPlaceholder::sigma_parse(r)?;
                    if r.substitute_placeholders() {
                        // ConstantPlaceholder itself can be created only if a corresponding
                        // constant is in the constant_store, thus unwrap() is safe here
                        let c = r.constant_store().get(cp.id).unwrap();
                        Ok(Expr::Const(c.clone().into()))
                    } else {
                        Ok(Expr::ConstPlaceholder(cp.into()))
                    }
                }
                OpCode::HEIGHT => Ok(Expr::GlobalVars(GlobalVars::Height.into())),
                OpCode::SELF_BOX => Ok(Expr::GlobalVars(GlobalVars::SelfBox.into())),
                OpCode::INPUTS => Ok(Expr::GlobalVars(GlobalVars::Inputs.into())),
                OpCode::OUTPUTS => Ok(Expr::GlobalVars(GlobalVars::Outputs.into())),
                OpCode::PROPERTY_CALL => {
                    Ok(Expr::ProperyCall(PropertyCall::sigma_parse(r)?.into()))
                }
                OpCode::METHOD_CALL => Ok(Expr::MethodCall(MethodCall::sigma_parse(r)?.into())),
                OpCode::CONTEXT => Ok(Expr::Context),
                OpCode::OPTION_GET => Ok(Box::new(OptionGet::sigma_parse(r)?).into()),
                OpCode::EXTRACT_REGISTER_AS => {
                    Ok(Box::new(ExtractRegisterAs::sigma_parse(r)?).into())
                }
                OpCode::EQ => Ok(bin_op_sigma_parse(LogicOp::Eq.into(), r)?),
                OpCode::NEQ => Ok(bin_op_sigma_parse(LogicOp::NEq.into(), r)?),
                OpCode::BLOCK_VALUE => Ok(Expr::BlockValue(BlockValue::sigma_parse(r)?.into())),
                OpCode::FUNC_VALUE => Ok(Expr::FuncValue(FuncValue::sigma_parse(r)?.into())),
                OpCode::VAL_DEF => Ok(Expr::ValDef(ValDef::sigma_parse(r)?.into())),
                OpCode::VAL_USE => Ok(Expr::ValUse(ValUse::sigma_parse(r)?.into())),
                OpCode::EXTRACT_AMOUNT => Ok(Expr::ExtractAmount(ExtractAmount::sigma_parse(r)?)),
                OpCode::SELECT_FIELD => Ok(Expr::SelectField(SelectField::sigma_parse(r)?)),
                o => Err(SerializationError::NotImplementedOpCode(o.value())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Expr>()) {
            dbg!(&v);
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}

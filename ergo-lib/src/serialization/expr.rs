use super::bin_op::bin_op_sigma_parse;
use super::bin_op::bin_op_sigma_serialize;
use super::{op_code::OpCode, sigma_byte_writer::SigmaByteWrite};
use crate::ast::and::And;
use crate::ast::apply::Apply;
use crate::ast::bin_op::ArithOp;
use crate::ast::bin_op::RelationOp;
use crate::ast::block::BlockValue;
use crate::ast::calc_blake2b256::CalcBlake2b256;
use crate::ast::coll_fold::Fold;
use crate::ast::collection::bool_const_coll_sigma_parse;
use crate::ast::collection::coll_sigma_parse;
use crate::ast::collection::coll_sigma_serialize;
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
                    let ph = cs.put(c.clone());
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
                    Expr::Apply(op) => op.sigma_serialize(w),
                    Expr::ExtractAmount(op) => op.sigma_serialize(w),
                    Expr::SelectField(op) => op.sigma_serialize(w),
                    Expr::CalcBlake2b256(op) => op.sigma_serialize(w),
                    Expr::Collection(op) => coll_sigma_serialize(op, w),
                    Expr::And(op) => op.sigma_serialize(w),
                    Expr::Const(_) => panic!("unexpected constant"), // handled in the code above (external match)
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
            Ok(Expr::Const(constant))
        } else {
            let op_code = OpCode::sigma_parse(r)?;
            match op_code {
                OpCode::FOLD => Ok(Fold::sigma_parse(r)?.into()),
                ConstantPlaceholder::OP_CODE => {
                    let cp = ConstantPlaceholder::sigma_parse(r)?;
                    if r.substitute_placeholders() {
                        // ConstantPlaceholder itself can be created only if a corresponding
                        // constant is in the constant_store, thus unwrap() is safe here
                        let c = r.constant_store().get(cp.id).unwrap();
                        Ok(Expr::Const(c.clone()))
                    } else {
                        Ok(Expr::ConstPlaceholder(cp))
                    }
                }
                OpCode::HEIGHT => Ok(Expr::GlobalVars(GlobalVars::Height)),
                OpCode::SELF_BOX => Ok(Expr::GlobalVars(GlobalVars::SelfBox)),
                OpCode::INPUTS => Ok(Expr::GlobalVars(GlobalVars::Inputs)),
                OpCode::OUTPUTS => Ok(Expr::GlobalVars(GlobalVars::Outputs)),
                OpCode::PROPERTY_CALL => Ok(Expr::ProperyCall(PropertyCall::sigma_parse(r)?)),
                OpCode::METHOD_CALL => Ok(Expr::MethodCall(MethodCall::sigma_parse(r)?)),
                OpCode::CONTEXT => Ok(Expr::Context),
                OpCode::OPTION_GET => Ok(OptionGet::sigma_parse(r)?.into()),
                OpCode::EXTRACT_REGISTER_AS => Ok(ExtractRegisterAs::sigma_parse(r)?.into()),
                OpCode::EQ => Ok(bin_op_sigma_parse(RelationOp::Eq.into(), r)?),
                OpCode::NEQ => Ok(bin_op_sigma_parse(RelationOp::NEq.into(), r)?),
                OpCode::GT => Ok(bin_op_sigma_parse(RelationOp::GT.into(), r)?),
                OpCode::LT => Ok(bin_op_sigma_parse(RelationOp::LT.into(), r)?),
                OpCode::GE => Ok(bin_op_sigma_parse(RelationOp::GE.into(), r)?),
                OpCode::LE => Ok(bin_op_sigma_parse(RelationOp::LE.into(), r)?),
                OpCode::PLUS => Ok(bin_op_sigma_parse(ArithOp::Plus.into(), r)?),
                OpCode::MINUS => Ok(bin_op_sigma_parse(ArithOp::Minus.into(), r)?),
                OpCode::MULTIPLY => Ok(bin_op_sigma_parse(ArithOp::Multiply.into(), r)?),
                OpCode::DIVISION => Ok(bin_op_sigma_parse(ArithOp::Divide.into(), r)?),
                OpCode::BLOCK_VALUE => Ok(Expr::BlockValue(BlockValue::sigma_parse(r)?)),
                OpCode::FUNC_VALUE => Ok(Expr::FuncValue(FuncValue::sigma_parse(r)?)),
                OpCode::APPLY => Ok(Expr::Apply(Apply::sigma_parse(r)?)),
                OpCode::VAL_DEF => Ok(Expr::ValDef(ValDef::sigma_parse(r)?)),
                OpCode::VAL_USE => Ok(Expr::ValUse(ValUse::sigma_parse(r)?)),
                OpCode::EXTRACT_AMOUNT => Ok(Expr::ExtractAmount(ExtractAmount::sigma_parse(r)?)),
                OpCode::SELECT_FIELD => Ok(Expr::SelectField(SelectField::sigma_parse(r)?)),
                OpCode::CALC_BLAKE2B256 => Ok(CalcBlake2b256::sigma_parse(r)?.into()),
                And::OP_CODE => Ok(And::sigma_parse(r)?.into()),
                OpCode::COLL => Ok(coll_sigma_parse(r)?.into()),
                OpCode::COLL_OF_BOOL_CONST => Ok(bool_const_coll_sigma_parse(r)?.into()),
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

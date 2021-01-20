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
use crate::types::sfunc::SFunc;
use crate::types::stype::SType;

use super::expr::Expr;
use super::val_def::ValId;
use super::value::Value;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct FuncArg {
    pub idx: ValId,
    pub tpe: SType,
}

impl SigmaSerializable for FuncArg {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.idx.sigma_serialize(w)?;
        self.tpe.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let idx = ValId::sigma_parse(r)?;
        let tpe = SType::sigma_parse(r)?;
        Ok(FuncArg { idx, tpe })
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct FuncValue {
    args: Vec<FuncArg>,
    body: Box<Expr>,
    tpe: SType,
}

impl FuncValue {
    pub fn new(args: Vec<FuncArg>, body: Expr) -> Self {
        let t_dom = args.iter().map(|fa| fa.tpe.clone()).collect();
        let t_range = body.tpe();
        let tpe = SType::SFunc(SFunc {
            t_dom,
            t_range: Box::new(t_range),
            tpe_params: vec![],
        });
        FuncValue {
            args,
            body: Box::new(body),
            tpe,
        }
    }

    pub fn args(&self) -> &[FuncArg] {
        self.args.as_ref()
    }

    pub fn body(&self) -> &Expr {
        &self.body
    }

    pub fn tpe(&self) -> SType {
        self.tpe.clone()
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::FUNC_VALUE
    }
}

impl Evaluable for FuncValue {
    fn eval(&self, _env: &Env, _ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(Value::FuncValue(Box::new(self.clone())))
    }
}

impl SigmaSerializable for FuncValue {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.args.sigma_serialize(w)?;
        self.body.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let args = Vec::<FuncArg>::sigma_parse(r)?;
        args.iter()
            .for_each(|a| r.val_def_type_store().insert(a.idx, a.tpe.clone()));
        let body = Expr::sigma_parse(r)?;
        Ok(FuncValue::new(args, body))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use super::*;

    impl Arbitrary for FuncValue {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<FuncArg>(), 1..10))
                .prop_map(|(body, args)| Self::new(args, body))
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(func_value in any::<FuncValue>()) {
            let e = Expr::FuncValue(func_value);
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}

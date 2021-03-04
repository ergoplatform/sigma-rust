//! Application of function
use std::io;

use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Application of function `func` to given arguments `args`
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Apply {
    /// Function
    pub func: Box<Expr>,
    /// Arguments
    pub args: Vec<Expr>,
}

impl Apply {
    /// Create new object, returns an error if any of the requirements failed
    pub fn new(func: Expr, args: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        let func = match func.tpe() {
            SType::SColl(_) => Ok(func),
            SType::SFunc(sfunc) => {
                let arg_types: Vec<SType> = args.iter().map(|a| a.tpe()).collect();
                if sfunc.t_dom != arg_types {
                    Err(InvalidArgumentError(format!(
                        "Expected args: {0:?}, got: {1:?}",
                        sfunc.t_dom, args
                    )))
                } else {
                    Ok(func)
                }
            }
            _ => Err(InvalidArgumentError(format!(
                "unexpected Apply::func: {0:?}",
                func.tpe(),
            ))),
        }?;
        Ok(Apply {
            func: Box::new(func),
            args,
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        match self.func.tpe() {
            SType::SColl(_) => todo!(),
            SType::SFunc(f) => *f.t_range,
            _ => panic!("unexpected Apply::func: {0:?}", self.func.tpe()),
        }
    }

    pub(crate) fn op_code(&self) -> OpCode {
        OpCode::APPLY
    }
}

impl SigmaSerializable for Apply {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.func.sigma_serialize(w)?;
        self.args.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let func = Expr::sigma_parse(r)?;
        let args = Vec::<Expr>::sigma_parse(r)?;
        Ok(Apply::new(func, args)?)
    }
}

#[cfg(test)]
mod tests {

    use crate::mir::func_value::*;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Apply {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<Expr>(), 1..10))
                .prop_map(|(body, args)| {
                    let func = Box::new(
                        FuncValue::new(
                            args.iter()
                                .enumerate()
                                .map(|(idx, arg)| FuncArg {
                                    idx: (idx as u32).into(),
                                    tpe: arg.tpe(),
                                })
                                .collect(),
                            body,
                        )
                        .into(),
                    );
                    Self { func, args }
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Apply>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }
    }
}

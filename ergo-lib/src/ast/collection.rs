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

use super::constant::TryExtractFromError;
use super::constant::TryExtractInto;
use super::expr::Expr;
use super::expr::InvalidArgumentError;
use super::value::CollKind;
use super::value::CollPrim;
use super::value::Value;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Collection {
    BoolConstants(Vec<bool>),
    Exprs { elem_tpe: SType, items: Vec<Expr> },
}

impl Collection {
    pub fn new(elem_tpe: SType, items: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        if !items.iter().all(|i| i.tpe() == elem_tpe) {
            return Err(InvalidArgumentError(format!(
                "expected items to be of the same type {0:?}, got {1:?}",
                elem_tpe, items
            )));
        }
        if elem_tpe == SType::SBoolean {
            let maybe_bools: Result<Vec<bool>, TryExtractFromError> = items
                .clone()
                .into_iter()
                .map(|i| i.try_extract_into::<bool>())
                .collect();
            match maybe_bools {
                Ok(bools) => Ok(Collection::BoolConstants(bools)),
                Err(_) => Ok(Collection::Exprs { elem_tpe, items }),
            }
        } else {
            Ok(Collection::Exprs { elem_tpe, items })
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(
            match self {
                Collection::BoolConstants(_) => SType::SBoolean,
                Collection::Exprs { elem_tpe, items: _ } => elem_tpe.clone(),
            }
            .into(),
        )
    }

    pub fn op_code(&self) -> OpCode {
        match self {
            Collection::BoolConstants(_) => OpCode::COLL_OF_BOOL_CONST,
            Collection::Exprs { .. } => OpCode::COLL,
        }
    }
}

impl Evaluable for Collection {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        Ok(match self {
            Collection::BoolConstants(bools) => bools.clone().into(),
            Collection::Exprs { elem_tpe, items } => {
                let items_v: Result<Vec<Value>, EvalError> =
                    items.iter().map(|i| i.eval(env, ctx)).collect();
                match elem_tpe {
                    SType::SByte => {
                        let bytes: Result<Vec<i8>, TryExtractFromError> = items_v?
                            .into_iter()
                            .map(|i| i.try_extract_into::<i8>())
                            .collect();
                        Value::Coll(CollKind::Primitive(CollPrim::CollByte(bytes?)))
                    }
                    _ => Value::Coll(CollKind::NonPrimitive {
                        elem_tpe: elem_tpe.clone(),
                        v: items_v?,
                    }),
                }
            }
        })
    }
}

pub fn coll_sigma_serialize<W: SigmaByteWrite>(
    coll: &Collection,
    w: &mut W,
) -> Result<(), std::io::Error> {
    match coll {
        Collection::BoolConstants(bools) => {
            w.put_u16(bools.len() as u16)?;
            w.put_bits(bools.as_slice())
        }
        Collection::Exprs { elem_tpe, items } => {
            w.put_u16(items.len() as u16)?;
            elem_tpe.sigma_serialize(w)?;
            items.iter().try_for_each(|i| i.sigma_serialize(w))
        }
    }
}

pub fn coll_sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Collection, SerializationError> {
    let items_count = r.get_u16()?;
    let elem_tpe = SType::sigma_parse(r)?;
    let mut items = Vec::with_capacity(items_count as usize);
    for _ in 0..items_count {
        items.push(Expr::sigma_parse(r)?);
    }
    Ok(Collection::Exprs { elem_tpe, items })
}

pub fn bool_const_coll_sigma_parse<R: SigmaByteRead>(
    r: &mut R,
) -> Result<Collection, SerializationError> {
    let items_count = r.get_u16()?;
    let bools = r.get_bits(items_count as usize)?;
    Ok(Collection::BoolConstants(bools))
}

#[cfg(test)]
mod tests {
    use crate::ast::constant::Constant;
    use crate::ast::expr::tests::ArbExprParams;
    use crate::eval::tests::eval_out_wo_ctx;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;
    use proptest::collection::*;
    use proptest::prelude::*;

    impl Arbitrary for Collection {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ArbExprParams;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                vec(any_with::<Expr>(args.clone()), 0..19),
                vec(
                    any_with::<Constant>(args.tpe.clone()).prop_map_into(),
                    0..19
                )
            ]
            .prop_map(move |items| Self::new(args.clone().tpe, items).unwrap())
            .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Collection>()) {
            dbg!(&v);
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn ser_roundtrip_bool_const(v in any_with::<Collection>(ArbExprParams{tpe: SType::SBoolean, depth: 0})) {
            dbg!(&v);
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn eval_byte_coll(bytes in any::<Vec<i8>>()) {
            let value: Value = bytes.clone().into();
            let exprs: Vec<Expr> = bytes.into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SByte, exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Value>(&coll);
            prop_assert_eq!(res, value);
        }

        #[test]
        fn eval_bool_coll(bools in any::<Vec<bool>>()) {
            let exprs: Vec<Expr> = bools.clone().into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SBoolean, exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Vec<bool>>(&coll);
            prop_assert_eq!(res, bools);
        }

        #[test]
        fn eval_long_coll(longs in any::<Vec<i64>>()) {
            let exprs: Vec<Expr> = longs.clone().into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SLong, exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Vec<i64>>(&coll);
            prop_assert_eq!(res, longs);
        }

        #[test]
        fn eval_bytes_coll_coll(bb in any::<Vec<Vec<i8>>>()) {
            let exprs: Vec<Expr> = bb.clone().into_iter().map(|b| Expr::Const(b.into())).collect();
            let coll: Expr = Collection::new(SType::SColl(SType::SByte.into()), exprs).unwrap().into();
            let res = eval_out_wo_ctx::<Vec<Vec<i8>>>(&coll);
            prop_assert_eq!(res, bb);
        }
    }
}

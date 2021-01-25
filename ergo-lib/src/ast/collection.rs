use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::constant::Constant;
use super::constant::TryExtractInto;
use super::expr::Expr;
use super::expr::InvalidArgumentError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    elem_tpe: SType,
    // todo: use enum (exprs vs. bools)
    items: Vec<Expr>,
    is_bool_const_coll: bool,
}

impl Collection {
    pub fn new(elem_tpe: SType, items: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        if items.iter().all(|i| i.tpe() == SType::SBoolean) {
            let is_bool_const_coll = elem_tpe == SType::SBoolean
                && items.iter().all(|i| {
                    matches!(
                        i,
                        Expr::Const(Constant {
                            tpe: SType::SBoolean,
                            v: _,
                        })
                    )
                });
            Ok(Collection {
                elem_tpe,
                items,
                is_bool_const_coll,
            })
        } else {
            Err(InvalidArgumentError(format!(
                "expected items to be of the same type {0:?}, got {1:?}",
                elem_tpe, items
            )))
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(self.elem_tpe.clone().into())
    }

    pub fn op_code(&self) -> OpCode {
        if self.is_bool_const_coll {
            OpCode::COLL_DECL_BOOL_CONST
        } else {
            OpCode::COLL_DECL
        }
    }
}

pub fn coll_sigma_serialize<W: SigmaByteWrite>(
    coll: &Collection,
    w: &mut W,
) -> Result<(), std::io::Error> {
    w.put_u16(coll.items.len() as u16)?;
    if coll.is_bool_const_coll {
        let bools: Vec<bool> = coll
            .clone()
            .items
            .into_iter()
            .map(|expr| expr.try_extract_into::<bool>().unwrap())
            .collect();
        w.put_bits(bools.as_slice())
    } else {
        coll.elem_tpe.sigma_serialize(w)?;
        coll.items.iter().try_for_each(|i| i.sigma_serialize(w))
    }
}

pub fn coll_sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Collection, SerializationError> {
    let items_count = r.get_u16()?;
    let elem_tpe = SType::sigma_parse(r)?;
    let mut items = Vec::with_capacity(items_count as usize);
    for _ in 0..items_count {
        items.push(Expr::sigma_parse(r)?);
    }
    Ok(Collection::new(elem_tpe, items)?)
}

pub fn bool_const_coll_sigma_parse<R: SigmaByteRead>(
    r: &mut R,
) -> Result<Collection, SerializationError> {
    let items_count = r.get_u16()?;
    let bools = r.get_bits(items_count as usize)?;
    let items = bools.into_iter().map(|b| Expr::Const(b.into())).collect();
    Ok(Collection::new(SType::SBoolean, items)?)
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::tests::ArbExprParams;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;
    use proptest::collection::*;
    use proptest::prelude::*;

    impl Arbitrary for Collection {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ArbExprParams;

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                vec(
                    any_with::<Expr>(ArbExprParams {
                        tpe: args.clone().tpe,
                        depth: args.depth,
                    }),
                    0..19
                ),
                vec(
                    any_with::<Constant>(args.clone().tpe).prop_map_into(),
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
    }
}

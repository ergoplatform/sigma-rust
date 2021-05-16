use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::constant::Constant;
use super::constant::TryExtractFromError;
use super::constant::TryExtractInto;
use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Collection of elements
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Collection {
    /// Special representation for an array of boolean constants
    BoolConstants(Vec<bool>),
    /// Colllection of elements, where each element is an expression
    Exprs {
        /// Element type
        elem_tpe: SType,
        /// Elements
        items: Vec<Expr>,
    },
}

impl Collection {
    /// Create new object, returns an error if any of the requirements failed
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
                .map(|i| i.try_extract_into::<Constant>()?.try_extract_into::<bool>())
                .collect();
            match maybe_bools {
                Ok(bools) => Ok(Collection::BoolConstants(bools)),
                Err(_) => Ok(Collection::Exprs { elem_tpe, items }),
            }
        } else {
            Ok(Collection::Exprs { elem_tpe, items })
        }
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SColl(
            match self {
                Collection::BoolConstants(_) => SType::SBoolean,
                Collection::Exprs { elem_tpe, items: _ } => elem_tpe.clone(),
            }
            .into(),
        )
    }
}

impl HasOpCode for Collection {
    fn op_code(&self) -> OpCode {
        match self {
            Collection::BoolConstants(_) => OpCode::COLL_OF_BOOL_CONST,
            Collection::Exprs { .. } => OpCode::COLL,
        }
    }
}

pub(crate) fn coll_sigma_serialize<W: SigmaByteWrite>(
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

pub(crate) fn coll_sigma_parse<R: SigmaByteRead>(
    r: &mut R,
) -> Result<Collection, SerializationError> {
    let items_count = r.get_u16()?;
    let elem_tpe = SType::sigma_parse(r)?;
    let mut items = Vec::with_capacity(items_count as usize);
    for _ in 0..items_count {
        items.push(Expr::sigma_parse(r)?);
    }
    Ok(Collection::Exprs { elem_tpe, items })
}

pub(crate) fn bool_const_coll_sigma_parse<R: SigmaByteRead>(
    r: &mut R,
) -> Result<Collection, SerializationError> {
    let items_count = r.get_u16()?;
    let bools = r.get_bits(items_count as usize)?;
    Ok(Collection::BoolConstants(bools))
}

#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
/// Arbitrary impl
mod arbitrary {
    use crate::mir::constant::arbitrary::ArbConstantParams;
    use crate::mir::expr::arbitrary::ArbExprParams;

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
                    any_with::<Constant>(ArbConstantParams::Exact(args.tpe.clone()))
                        .prop_map_into(),
                    0..19
                )
            ]
            .prop_map(move |items| Self::new(args.clone().tpe, items).unwrap())
            .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::mir::expr::arbitrary::ArbExprParams;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<Collection>()) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

        #[test]
        fn ser_roundtrip_bool_const(v in any_with::<Collection>(ArbExprParams{tpe: SType::SBoolean, depth: 0})) {
            let expr: Expr = v.into();
            prop_assert_eq![sigma_serialize_roundtrip(&expr), expr];
        }

    }
}

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

use super::constant::TryExtractInto;
use super::expr::Expr;
use super::val_def::ValDef;
use super::value::Value;

/** The order of ValDefs in the block is used to assign ids to ValUse(id) nodes
 * For all i: items(i).id == {number of ValDefs preceded in a graph} with respect to topological order.
 * Specific topological order doesn't really matter, what is important is to preserve semantic linkage
 * between ValUse(id) and ValDef with the corresponding id.
 * This convention allow to valid serializing ids because we always serializing and deserializing
 * in a fixed well defined order.
 */
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlockValue {
    pub items: Vec<Expr>,
    pub result: Box<Expr>,
}

impl BlockValue {
    pub fn tpe(&self) -> SType {
        self.result.tpe()
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::BLOCK_VALUE
    }
}

impl Evaluable for BlockValue {
    fn eval(&self, env: &Env, ctx: &mut EvalContext) -> Result<Value, EvalError> {
        let mut cur_env = env.clone();
        for i in self.items.iter() {
            let val_def = i.clone().try_extract_into::<ValDef>()?;
            let v: Value = val_def.rhs.eval(&cur_env, ctx)?;
            cur_env.insert(val_def.id, v);
        }
        self.result.eval(&cur_env, ctx)
    }
}

impl SigmaSerializable for BlockValue {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        self.items.sigma_serialize(w)?;
        self.result.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let items = Vec::<Expr>::sigma_parse(r)?;
        dbg!(&items);
        let result = Expr::sigma_parse(r)?;
        Ok(BlockValue {
            items,
            result: Box::new(result),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::block::BlockValue;
    use crate::ast::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;

    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for BlockValue {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<Expr>(), 0..10))
                .prop_map(|(result, items)| Self {
                    items,
                    result: Box::new(result),
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            let e = Expr::BlockValue(block);
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}

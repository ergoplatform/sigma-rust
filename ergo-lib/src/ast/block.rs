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
    pub items: Vec<ValDef>,
    pub result: Expr,
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
            let v: Value = i.rhs.eval(&cur_env, ctx)?;
            cur_env.insert(i.id, v);
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
        let items = Vec::<ValDef>::sigma_parse(r)?;
        let result = Expr::sigma_parse(r)?;
        Ok(BlockValue { items, result })
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::block::BlockValue;
    use crate::ast::expr::Expr;
    use crate::ast::val_def::ValDef;
    use crate::serialization::sigma_serialize_roundtrip;

    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for BlockValue {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Expr>(), vec(any::<ValDef>(), 0..10))
                .prop_map(|(result, items)| Self { items, result })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&block), block];
        }
    }
}

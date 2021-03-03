use std::io;

use super::expr::Expr;
use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

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

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
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
}

#[cfg(test)]
pub mod tests {
    use crate::mir::block::BlockValue;
    use crate::mir::expr::Expr;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(block in any::<BlockValue>()) {
            let e = Expr::BlockValue(block);
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}

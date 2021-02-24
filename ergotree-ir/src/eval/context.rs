use std::rc::Rc;

use crate::ir_ergo_box::IrBoxId;
use crate::ir_ergo_box::IrErgoBoxArena;

#[derive(Debug)]
pub struct Context {
    pub box_arena: Rc<dyn IrErgoBoxArena>,
    pub height: i32,
    pub self_box: IrBoxId,
    pub outputs: Vec<IrBoxId>,
    pub data_inputs: Vec<IrBoxId>,
}

#[cfg(feature = "arbitrary")]
pub mod arbitrary {
    use super::*;
    // use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Context {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            todo!()
            // (
            //     0..i32::MAX,
            //     any::<ErgoBox>(),
            //     vec(any::<ErgoBox>(), 1..3),
            //     vec(any::<ErgoBox>(), 0..3),
            // )
            //     .prop_map(|(height, self_box, outputs, data_inputs)| Self {
            //         box_arena,
            //         height,
            //         self_box,
            //         outputs,
            //         data_inputs,
            //     })
            //     .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {}

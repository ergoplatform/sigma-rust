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
    use std::collections::HashMap;

    use crate::ir_ergo_box::IrErgoBoxDummy;
    use crate::ir_ergo_box::IrErgoBoxDummyArena;

    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Context {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                0..i32::MAX,
                any::<IrErgoBoxDummy>(),
                vec(any::<IrErgoBoxDummy>(), 1..3),
                vec(any::<IrErgoBoxDummy>(), 0..3),
            )
                .prop_map(|(height, self_box, outputs, data_inputs)| {
                    let mut m = HashMap::new();
                    m.insert(self_box.id, self_box);
                    outputs.into_iter().for_each(|b| {
                        m.insert(b.id, b);
                    });
                    data_inputs.into_iter().for_each(|b| {
                        m.insert(b.id, b);
                    });
                    let box_arena = IrErgoBoxDummyArena(m);
                    Self {
                        box_arena: Rc::new(box_arena) as Rc<dyn IrErgoBoxArena>,
                        height,
                        self_box: self_box.id,
                        outputs: outputs.iter().map(|b| b.id).collect(),
                        data_inputs: data_inputs.iter().map(|b| b.id).collect(),
                    }
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {}

mod ir_ergo_box_dummy;

use std::rc::Rc;

use ergotree_ir::ir_ergo_box::IrBoxId;
use ergotree_ir::ir_ergo_box::IrErgoBoxArena;

#[derive(Debug)]
pub struct Context {
    pub box_arena: Rc<dyn IrErgoBoxArena>,
    pub height: i32,
    pub self_box: IrBoxId,
    pub outputs: Vec<IrBoxId>,
    pub data_inputs: Vec<IrBoxId>,
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use std::collections::HashMap;

    use super::ir_ergo_box_dummy::*;
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
                    let self_box_id = self_box.id.clone();
                    let outputs_ids = outputs.iter().map(|b| b.id.clone()).collect();
                    let data_inputs_ids = data_inputs.iter().map(|b| b.id.clone()).collect();
                    let mut m = HashMap::new();
                    m.insert(self_box_id.clone(), self_box);
                    outputs.into_iter().for_each(|b| {
                        m.insert(b.id.clone(), b);
                    });
                    data_inputs.into_iter().for_each(|b| {
                        m.insert(b.id.clone(), b);
                    });
                    let box_arena = IrErgoBoxDummyArena(m);
                    Self {
                        box_arena: Rc::new(box_arena) as Rc<dyn IrErgoBoxArena>,
                        height,
                        self_box: self_box_id,
                        outputs: outputs_ids,
                        data_inputs: data_inputs_ids,
                    }
                })
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
mod tests {}

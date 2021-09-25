pub(crate) mod ir_ergo_box_dummy;

use std::rc::Rc;

use crate::sigma_protocol::prover::ContextExtension;
use ergotree_ir::ir_ergo_box::IrBoxId;
use ergotree_ir::ir_ergo_box::IrErgoBoxArena;
use ergotree_ir::mir::header::PreHeader;

/// Interpreter's context (blockchain state)
#[derive(Debug)]
pub struct Context {
    /// Arena with all boxes (from self, inputs, outputs, data_inputs)
    pub box_arena: Rc<dyn IrErgoBoxArena>,
    /// Current height
    pub height: u32,
    /// Box that contains the script we're evaluating (from spending transaction inputs)
    pub self_box: IrBoxId,
    /// Spending transaction outputs
    pub outputs: Vec<IrBoxId>,
    /// Spending transaction data inputs
    pub data_inputs: Vec<IrBoxId>,
    /// Spending transaction inputs
    pub inputs: Vec<IrBoxId>,
    /// Pre header of current block
    pub pre_header: PreHeader,
    /// prover-defined key-value pairs, that may be used inside a script
    pub extension: ContextExtension,
}

impl Context {
    /// Return a new Context with given context extension
    pub fn with_extension(self, ext: ContextExtension) -> Self {
        Context {
            extension: ext,
            ..self
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use std::collections::HashMap;

    use super::ir_ergo_box_dummy::*;
    use super::*;
    use ergotree_ir::ir_ergo_box::IrErgoBox;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Context {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                0..i32::MAX as u32,
                any::<IrErgoBoxDummy>(),
                vec(any::<IrErgoBoxDummy>(), 1..3),
                vec(any::<IrErgoBoxDummy>(), 1..3),
                vec(any::<IrErgoBoxDummy>(), 0..3),
                any::<PreHeader>(),
                any::<ContextExtension>(),
            )
                .prop_map(
                    |(height, self_box, outputs, inputs, data_inputs, pre_header, extension)| {
                        let self_box_id = self_box.id();
                        let outputs_ids = outputs.iter().map(|b| b.id()).collect();
                        let inputs_ids = inputs.iter().map(|b| b.id()).collect();
                        let data_inputs_ids = data_inputs.iter().map(|b| b.id()).collect();
                        let mut m = HashMap::new();
                        m.insert(self_box_id.clone(), self_box);
                        outputs.into_iter().for_each(|b| {
                            m.insert(b.id(), b);
                        });
                        inputs.into_iter().for_each(|b| {
                            m.insert(b.id(), b);
                        });
                        data_inputs.into_iter().for_each(|b| {
                            m.insert(b.id(), b);
                        });
                        let box_arena = IrErgoBoxDummyArena(m);
                        Self {
                            box_arena: Rc::new(box_arena) as Rc<dyn IrErgoBoxArena>,
                            height,
                            self_box: self_box_id,
                            outputs: outputs_ids,
                            data_inputs: data_inputs_ids,
                            inputs: inputs_ids,
                            pre_header,
                            extension,
                        }
                    },
                )
                .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}

#[cfg(test)]
mod tests {}

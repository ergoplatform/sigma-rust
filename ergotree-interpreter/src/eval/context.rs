use std::rc::Rc;

use crate::sigma_protocol::prover::ContextExtension;
use ergotree_ir::chain::ergo_box::ErgoBox;
use ergotree_ir::chain::header::Header;
use ergotree_ir::chain::preheader::PreHeader;

/// Interpreter's context (blockchain state)
#[derive(Debug)]
pub struct Context {
    /// Current height
    pub height: u32,
    /// Box that contains the script we're evaluating (from spending transaction inputs)
    pub self_box: Rc<ErgoBox>,
    /// Spending transaction outputs
    pub outputs: Vec<Rc<ErgoBox>>,
    /// Spending transaction data inputs
    pub data_inputs: Vec<Rc<ErgoBox>>,
    /// Spending transaction inputs
    pub inputs: Vec<Rc<ErgoBox>>,
    /// Pre header of current block
    pub pre_header: PreHeader,
    /// Fixed number of last block headers in descending order (first header is the newest one)
    pub headers: [Header; 10],
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

    use super::*;
    use proptest::collection::vec;
    use proptest::prelude::*;

    impl Arbitrary for Context {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                0..i32::MAX as u32,
                any::<ErgoBox>(),
                vec(any::<ErgoBox>(), 1..3),
                vec(any::<ErgoBox>(), 1..3),
                vec(any::<ErgoBox>(), 0..3),
                any::<PreHeader>(),
                any::<ContextExtension>(),
                any::<[Header; 10]>(),
            )
                .prop_map(
                    |(
                        height,
                        self_box,
                        outputs,
                        inputs,
                        data_inputs,
                        pre_header,
                        extension,
                        headers,
                    )| {
                        Self {
                            height,
                            self_box: Rc::new(self_box),
                            outputs: outputs.into_iter().map(Rc::new).collect(),
                            data_inputs: data_inputs.into_iter().map(Rc::new).collect(),
                            inputs: inputs.into_iter().map(Rc::new).collect(),
                            pre_header,
                            extension,
                            headers,
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

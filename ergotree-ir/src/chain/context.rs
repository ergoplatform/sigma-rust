//! Context(blockchain) for the interpreter
use core::cell::Cell;

use crate::chain::ergo_box::ErgoBox;
use crate::mir::constant::Constant;
use crate::{chain::context_extension::ContextExtension, ergo_tree::ErgoTreeVersion};
use bounded_vec::BoundedVec;
use ergo_chain_types::{Header, PreHeader};

/// BoundedVec type for Tx inputs, output_candidates and outputs
pub type TxIoVec<T> = BoundedVec<T, 1, { i16::MAX as usize }>;

/// Interpreter's context (blockchain state)
#[derive(derive_more::Debug, Clone)]
pub struct Context<'ctx> {
    /// Current height
    pub height: u32,
    /// Box that contains the script we're evaluating (from spending transaction inputs)
    pub self_box: &'ctx ErgoBox,
    /// Spending transaction outputs
    pub outputs: &'ctx [ErgoBox],
    /// Spending transaction data inputs
    pub data_inputs: Option<TxIoVec<&'ctx ErgoBox>>,
    /// Spending transaction inputs
    pub inputs: TxIoVec<&'ctx ErgoBox>,
    /// Pre header of current block
    pub pre_header: PreHeader,
    /// Fixed number of last block headers in descending order (first header is the newest one)
    pub headers: [Header; 10],
    /// prover-defined key-value pairs, that may be used inside a script
    pub extension: &'ctx ContextExtension,
    /// ergo tree version
    pub tree_version: Cell<ErgoTreeVersion>,
    /// ContextExtension provider for inputs of transaction
    #[debug(skip)]
    pub extension_provider: &'ctx dyn ContextExtensionProvider,
    /// Constants from ErgoTree for lazy ConstPlaceholder resolution during evaluation.
    /// None when constants were already substituted (e.g. via proposition()).
    pub constants: Option<&'ctx [Constant]>,
}

impl<'ctx> Context<'ctx> {
    /// Return a new Context with given context extension
    pub fn with_extension(self, ext: &'ctx ContextExtension) -> Self {
        Context {
            extension: ext,
            ..self
        }
    }
    /// Activated script version corresponds to block version - 1
    pub fn activated_script_version(&self) -> ErgoTreeVersion {
        ErgoTreeVersion::from(self.pre_header.version.saturating_sub(1))
    }
    /// Version of ergotree being evaluated under context
    pub fn tree_version(&self) -> ErgoTreeVersion {
        self.tree_version.get()
    }

    /// Create a copy of this context with constants set for lazy ConstPlaceholder resolution.
    /// The returned Context may have a shorter lifetime 'a to accommodate
    /// the constants reference alongside the existing borrowed fields.
    pub fn with_constants<'a>(&self, constants: &'a [Constant]) -> Context<'a>
    where
        'ctx: 'a,
    {
        Context {
            constants: Some(constants),
            ..self.clone()
        }
    }
}

// Since `ErgoTransaction` is defined in ergo-lib, we can't use it directly, so instead we use this trait and impl it for all `ErgoTransaction`s
/// Provides access to [`ContextExtension`] of transaction inputs
pub trait ContextExtensionProvider {
    /// Returns a reference to [`ContextExtension`] of input at index
    fn context_extension(&self, input_index: usize) -> Option<&ContextExtension>;
}

#[cfg(feature = "arbitrary")]
#[doc(hidden)]
#[allow(clippy::unwrap_used, missing_docs)]
pub mod arbitrary {

    use super::*;
    use proptest::{collection::vec, option::of, prelude::*};

    pub struct DummyContextExtensionProvider(pub Vec<ContextExtension>);

    impl ContextExtensionProvider for DummyContextExtensionProvider {
        fn context_extension(&self, input_index: usize) -> Option<&ContextExtension> {
            self.0.get(input_index)
        }
    }

    impl Arbitrary for Context<'static> {
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let input_strategy = vec(any::<ErgoBox>(), 1..3).prop_flat_map(|input_boxes| {
                let len = input_boxes.len();
                (Just(input_boxes), vec(any::<ContextExtension>(), len..=len))
            });
            (
                0..i32::MAX as u32,
                vec(any::<ErgoBox>(), 1..3),
                input_strategy,
                of(vec(any::<ErgoBox>(), 1..3)),
                any::<PreHeader>(),
                any::<[Header; 10]>(),
            )
                .prop_map(
                    |(
                        height,
                        outputs,
                        (input_boxes, extensions),
                        data_inputs,
                        pre_header,
                        headers,
                    )| {
                        // Leak variables. Since this is only used for testing this is acceptable and avoids introducing a new type (ContextOwned)
                        Self {
                            height,
                            self_box: Box::leak(input_boxes[0].clone().into()),
                            outputs: Vec::leak(outputs),
                            data_inputs: data_inputs.map(|v| {
                                v.into_iter()
                                    .map(|i| &*Box::leak(Box::new(i)))
                                    .collect::<Vec<_>>()
                                    .try_into()
                                    .unwrap()
                            }),
                            inputs: input_boxes
                                .into_iter()
                                .map(|i| &*Box::leak(Box::new(i)))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                            pre_header,
                            extension: Box::leak(extensions[0].clone().into()),
                            headers,
                            tree_version: Default::default(),
                            extension_provider: Box::leak(
                                DummyContextExtensionProvider(extensions).into(),
                            ),
                            constants: None,
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

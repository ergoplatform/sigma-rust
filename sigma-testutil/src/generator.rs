use proptest::{arbitrary::Arbitrary, prelude::*};
use sigma_tree::ErgoTree;

#[derive(Debug)]
pub struct ErgoTreeArb(pub ErgoTree);

impl Arbitrary for ErgoTreeArb {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        todo!()
        // (any::<u32>(),)
        //     .prop_map(|_| Self {
        //         0: todo!(), //ErgoTree::from_proposition(Expr::Const(SigmaBoolean::ProveDlog()),
        //     })
        //     .boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

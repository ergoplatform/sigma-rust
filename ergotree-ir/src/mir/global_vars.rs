//! Global variables

use alloc::sync::Arc;
use core::fmt::Display;

use crate::has_opcode::HasOpCode;
use crate::serialization::op_code::OpCode;
use crate::traversable::impl_traversable_expr;
use crate::types::stype::SType;

use super::expr::Expr;

#[derive(PartialEq, Eq, Debug, Clone)]
/// Predefined global variables
pub enum GlobalVars {
    /// Tx inputs
    Inputs,
    /// Tx outputs
    Outputs,
    /// Current blockchain height
    Height,
    /// ErgoBox instance, which script is being evaluated
    SelfBox,
    /// When interpreted evaluates to a ByteArrayConstant built from Context.minerPubkey
    MinerPubKey,
    /// When interpreted evaluates to a AvlTreeConstant built from Context.lastBlockUtxoRoot
    LastBlockUtxoRootHash,
    /// GroupElement (EcPoint) generator
    GroupGenerator,
}

impl GlobalVars {
    /// Type
    pub fn tpe(&self) -> SType {
        match self {
            GlobalVars::Inputs => SType::SColl(Arc::new(SType::SBox)),
            GlobalVars::Outputs => SType::SColl(Arc::new(SType::SBox)),
            GlobalVars::Height => SType::SInt,
            GlobalVars::SelfBox => SType::SBox,
            GlobalVars::MinerPubKey => SType::SColl(Arc::new(SType::SByte)),
            GlobalVars::LastBlockUtxoRootHash => SType::SAvlTree,
            GlobalVars::GroupGenerator => SType::SGroupElement,
        }
    }
}

impl HasOpCode for GlobalVars {
    /// Op code (serialization)
    fn op_code(&self) -> OpCode {
        match self {
            GlobalVars::SelfBox => OpCode::SELF_BOX,
            GlobalVars::Inputs => OpCode::INPUTS,
            GlobalVars::Outputs => OpCode::OUTPUTS,
            GlobalVars::Height => OpCode::HEIGHT,
            GlobalVars::MinerPubKey => OpCode::MINER_PUBKEY,
            GlobalVars::LastBlockUtxoRootHash => OpCode::LAST_BLOCK_UTXO_ROOT_HASH,
            GlobalVars::GroupGenerator => OpCode::GROUP_GENERATOR,
        }
    }
}

impl Display for GlobalVars {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GlobalVars::SelfBox => write!(f, "SELF"),
            GlobalVars::Inputs => write!(f, "INPUTS"),
            GlobalVars::Outputs => write!(f, "OUTPUTS"),
            GlobalVars::Height => write!(f, "HEIGHT"),
            GlobalVars::MinerPubKey => write!(f, "MINER_PUBKEY"),
            GlobalVars::LastBlockUtxoRootHash => write!(f, "LastBlockUtxoRootHash"),
            GlobalVars::GroupGenerator => write!(f, "GROUP_GENERATOR"),
        }
    }
}

impl_traversable_expr!(GlobalVars);

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use proptest::prelude::*;

    impl Arbitrary for GlobalVars {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            use GlobalVars::*;
            prop_oneof![
                Just(Inputs),
                Just(Outputs),
                Just(Height),
                Just(SelfBox),
                Just(MinerPubKey),
                Just(LastBlockUtxoRootHash),
                Just(GroupGenerator)
            ]
            .boxed()
        }
    }
}

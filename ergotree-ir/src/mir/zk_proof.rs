use alloc::boxed::Box;

use crate::traversable::Traversable;
use crate::types::stype::SType;

use super::expr::Expr;
use super::expr::InvalidArgumentError;

/// Explicit Zero Knowledge scope wrapping a SigmaProp body.
///
/// Mirrors Scala's `sigma.ast.ZKProofBlock` (data/shared values.scala:1110).
/// Scala assigns `OpCodes.Undefined` (byte 0) and provides no serializer —
/// the node exists at the typer/AST layer only. The Scala compiler test
/// `ZKProof { sigmaProp(HEIGHT > 1000) }` is annotated
/// `testMissingCostingWOSerialization`, asserting that the compiler builds
/// the IR but the graph-builder (and serializer) reject it with
/// `GraphBuildingException`.
///
/// The Rust IR mirrors the parity exactly: the node can be constructed by
/// the compiler frontend and round-trips through type checking, but
/// serialization errors with `NotSupported` and evaluation errors with
/// `EvalError::Misc`. There is no `HasStaticOpCode` impl — the op-code byte
/// space is exhausted (XOR_OF = 255), and Scala also has no canonical op-code.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ZkProofBlock {
    /// SigmaProp body of the ZKProof scope.
    pub input: Box<Expr>,
}

impl ZkProofBlock {
    /// Type — always SBoolean (matches Scala's `tpe = SBoolean`).
    pub fn tpe(&self) -> SType {
        SType::SBoolean
    }

    /// Build a `ZkProofBlock` after checking the body type is `SSigmaProp`.
    pub fn try_build(input: Expr) -> Result<Self, InvalidArgumentError> {
        input.check_post_eval_tpe(&SType::SSigmaProp)?;
        Ok(ZkProofBlock {
            input: input.into(),
        })
    }
}

impl Traversable for ZkProofBlock {
    type Item = Expr;

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Expr> + 'a> {
        Box::new(core::iter::once(self.input.as_ref()))
    }

    fn children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut Expr> + 'a> {
        Box::new(core::iter::once(self.input.as_mut()))
    }
}

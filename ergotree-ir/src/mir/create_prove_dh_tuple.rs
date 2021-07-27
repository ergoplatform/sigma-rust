use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::types::stype::SType;

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::mir::expr::InvalidArgumentError;

/// Diffie-Hellman tuple.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateProveDhTuple {
    /// Group generator `g`
    pub g: Box<Expr>,
    /// Point `g^x`
    pub h: Box<Expr>,
    /// Point `g^y`
    pub u: Box<Expr>,
    /// Point `g^xy`
    pub v: Box<Expr>,
}

impl CreateProveDhTuple {
    /// Create ProveDHTuple from four points on elliptic curve
    pub fn new(g: Expr, h: Expr, u: Expr, v: Expr) -> Result<Self, InvalidArgumentError> {
        g.check_post_eval_tpe(&SType::SGroupElement)?;
        h.check_post_eval_tpe(&SType::SGroupElement)?;
        u.check_post_eval_tpe(&SType::SGroupElement)?;
        v.check_post_eval_tpe(&SType::SGroupElement)?;
        Ok(CreateProveDhTuple {
            g: g.into(),
            h: h.into(),
            u: u.into(),
            v: v.into(),
        })
    }

    /// Type
    pub fn tpe(&self) -> SType {
        SType::SSigmaProp
    }
}

impl HasStaticOpCode for CreateProveDhTuple {
    const OP_CODE: OpCode = OpCode::PROVE_DIFFIE_HELLMAN_TUPLE;
}

impl SigmaSerializable for CreateProveDhTuple {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.g.sigma_serialize(w)?;
        self.h.sigma_serialize(w)?;
        self.u.sigma_serialize(w)?;
        self.v.sigma_serialize(w)
    }

    #[allow(clippy::many_single_char_names)]
    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let g = Expr::sigma_parse(r)?.into();
        let h = Expr::sigma_parse(r)?.into();
        let u = Expr::sigma_parse(r)?.into();
        let v = Expr::sigma_parse(r)?.into();
        Ok(CreateProveDhTuple { g, h, u, v })
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod tests {
    use sigma_test_util::force_any_val_with;

    use crate::mir::constant::arbitrary::ArbConstantParams;
    use crate::mir::constant::Constant;
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    #[test]
    fn ser_roundtrip() {
        let e: Expr = CreateProveDhTuple::new(
            force_any_val_with::<Constant>(ArbConstantParams::Exact(SType::SGroupElement)).into(),
            force_any_val_with::<Constant>(ArbConstantParams::Exact(SType::SGroupElement)).into(),
            force_any_val_with::<Constant>(ArbConstantParams::Exact(SType::SGroupElement)).into(),
            force_any_val_with::<Constant>(ArbConstantParams::Exact(SType::SGroupElement)).into(),
        )
        .unwrap()
        .into();
        assert_eq![sigma_serialize_roundtrip(&e), e];
    }
}

use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;
use crate::has_opcode::HasStaticOpCode;
use crate::mir::expr::InvalidArgumentError;

/// Diffie-Hellman tuple.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CreateProveDhTuple {
    /// Group generator `g`
    pub gv: Box<Expr>,
    /// Point `g^x`
    pub hv: Box<Expr>,
    /// Point `g^y`
    pub uv: Box<Expr>,
    /// Point `g^xy`
    pub vv: Box<Expr>,
}

impl CreateProveDhTuple {
    /// Create ProveDHTuple from four points on elliptic curve
    pub fn new(gv: Expr, hv: Expr, uv: Expr, vv: Expr) -> Result<Self, InvalidArgumentError> {
        gv.check_post_eval_tpe(&SType::SGroupElement)?;
        hv.check_post_eval_tpe(&SType::SGroupElement)?;
        uv.check_post_eval_tpe(&SType::SGroupElement)?;
        vv.check_post_eval_tpe(&SType::SGroupElement)?;
        Ok(CreateProveDhTuple {
            gv: gv.into(),
            hv: hv.into(),
            uv: uv.into(),
            vv: vv.into(),
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
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        self.gv.sigma_serialize(w)?;
        self.hv.sigma_serialize(w)?;
        self.uv.sigma_serialize(w)?;
        self.vv.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let gv = Expr::sigma_parse(r)?.into();
        let hv = Expr::sigma_parse(r)?.into();
        let uv = Expr::sigma_parse(r)?.into();
        let vv = Expr::sigma_parse(r)?.into();
        Ok(CreateProveDhTuple { gv, hv, uv, vv })
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

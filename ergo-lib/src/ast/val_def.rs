use std::io::Error;

use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::expr::Expr;

extern crate derive_more;
use derive_more::From;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, From)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ValId(u32);

impl SigmaSerializable for ValId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        w.put_u32(self.0)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let id = r.get_u32()?;
        Ok(ValId(id))
    }
}

/** IR node for let-bound expressions `let x = rhs` which is ValDef.
 * These nodes are used to represent ErgoTrees after common sub-expression elimination.
 * This representation is more compact in serialized form.
 * @param id unique identifier of the variable in the current scope. */
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct ValDef {
    pub id: ValId,
    pub rhs: Expr,
}

impl ValDef {
    pub fn tpe(&self) -> SType {
        self.rhs.tpe()
    }

    pub fn op_code(&self) -> OpCode {
        OpCode::VAL_DEF
    }
}

impl SigmaSerializable for ValDef {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        self.id.sigma_serialize(w)?;
        self.rhs.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let id = ValId::sigma_parse(r)?;
        let rhs = Expr::sigma_parse(r)?;
        r.val_def_type_store().insert(id, rhs.tpe());
        Ok(ValDef { id, rhs })
    }
}

#[cfg(test)]
mod tests {
    use crate::serialization::sigma_serialize_roundtrip;

    use super::*;

    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ValDef>()) {
            let e = Expr::ValDef(v.into());
            prop_assert_eq![sigma_serialize_roundtrip(&e), e];
        }
    }
}

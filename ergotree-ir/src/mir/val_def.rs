use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use crate::serialization::SigmaSerializeResult;
use crate::traversable::impl_traversable_expr;
use crate::types::stype::SType;

use super::expr::Expr;

extern crate derive_more;
use alloc::boxed::Box;
use derive_more::Display;
use derive_more::From;

use crate::has_opcode::HasStaticOpCode;
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

/// Variable id
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, From, Display, Ord, PartialOrd)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct ValId(
    // Valid variable ids fit in a signed `Int` (`ValDef.id` is read with
    // `getUIntExact`); keep the generator in range so `ValDef` round-trips
    // through the bounded parse. `ValUse` accepts the wider wire range but its
    // ids reference a `ValDef`, so in-range generation stays representative.
    #[cfg_attr(feature = "arbitrary", proptest(strategy = "0u32..=(i32::MAX as u32)"))] pub u32,
);

impl ValId {
    pub(crate) fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> core3::io::Result<()> {
        w.put_u32(self.0)
    }

    pub(crate) fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = r.get_u32()?;
        Ok(ValId(id))
    }

    /// Parse a `ValId` with the `getUIntExact` bound the JVM `ValDefSerializer`
    /// applies to `ValDef.id`: the unsigned VLQ must fit in a signed `Int`
    /// (`<= 0x7fffffff`), else deserialization fails (JVM `toIntExact` throws
    /// `ArithmeticException`, before any eval). `ValUse`/`FuncValue` ids use the
    /// wrapping `getUInt().toInt` and deliberately stay on the plain
    /// `sigma_parse` — bounding them would over-reject.
    pub(crate) fn sigma_parse_exact<R: SigmaByteRead>(
        r: &mut R,
    ) -> Result<Self, SigmaParsingError> {
        let id = r.get_u32()?;
        if id > i32::MAX as u32 {
            return Err(SigmaParsingError::Misc(alloc::format!(
                "ValDef id {id} exceeds Int.MaxValue (getUIntExact)"
            )));
        }
        Ok(ValId(id))
    }
}

/** IR node for let-bound expressions `let x = rhs` which is ValDef.
 * These nodes are used to represent ErgoTrees after common sub-expression elimination.
 * This representation is more compact in serialized form.
 * @param id unique identifier of the variable in the current scope. */
#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub struct ValDef {
    /// Variable id
    pub id: ValId,
    /// Expr, bound to the variable
    pub rhs: Box<Expr>,
}

impl ValDef {
    /// Type
    pub fn tpe(&self) -> SType {
        self.rhs.tpe()
    }
}

impl HasStaticOpCode for ValDef {
    const OP_CODE: OpCode = OpCode::VAL_DEF;
}

impl SigmaSerializable for ValDef {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        self.id.sigma_serialize(w)?;
        self.rhs.sigma_serialize(w)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let id = ValId::sigma_parse_exact(r)?;
        let rhs = Expr::sigma_parse(r)?;
        r.val_def_type_store().insert(id, rhs.tpe());
        Ok(ValDef {
            id,
            rhs: Box::new(rhs),
        })
    }
}

impl_traversable_expr!(ValDef, boxed rhs);

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
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

    #[test]
    #[allow(clippy::unwrap_used)]
    fn valdef_id_getuintexact_bound() {
        // JVM `ValDefSerializer` reads `ValDef.id` with `getUIntExact`, so an id
        // VLQ above Int.MaxValue is rejected at deserialize. The inclusive max
        // 0x7fffffff (`ffffffff07`) binds; 0x80000000 (`8080808008`) errors.
        // ValUse keeps the wrapping `getUInt().toInt` read and still accepts it.
        use crate::serialization::sigma_byte_reader::from_bytes;
        let accept = [0xffu8, 0xff, 0xff, 0xff, 0x07];
        let reject = [0x80u8, 0x80, 0x80, 0x80, 0x08];
        assert_eq!(
            ValId::sigma_parse_exact(&mut from_bytes(&accept[..])).unwrap(),
            ValId(0x7fff_ffff)
        );
        assert!(ValId::sigma_parse_exact(&mut from_bytes(&reject[..])).is_err());
        assert_eq!(
            ValId::sigma_parse(&mut from_bytes(&reject[..])).unwrap(),
            ValId(0x8000_0000)
        );
    }
}

use crate::types::SType;
use sigma_ser::{
    serializer::{SerializationError, SigmaSerializable},
    vlq_encode,
};
use std::{io, ops::Add};
use vlq_encode::{ReadSigmaVlqExt, WriteSigmaVlqExt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCode(u8);

impl TypeCode {
    /// Type code of the last valid prim type so that (1 to LastPrimTypeCode) is a range of valid codes.
    pub const LAST_PRIM_TYPECODE: u8 = 8;

    /// Upper limit of the interval of valid type codes for primitive types
    pub const MAX_PRIM_TYPECODE: u8 = 11;
    pub const PRIM_RANGE: u8 = (TypeCode::MAX_PRIM_TYPECODE + 1);

    pub const SBOOLEAN: TypeCode = Self::new(1);
    pub const SBYTE: TypeCode = Self::new(2);
    pub const SSHORT: TypeCode = Self::new(3);
    pub const SINT: TypeCode = Self::new(4);
    pub const SLONG: TypeCode = Self::new(5);
    pub const SBIGINT: TypeCode = Self::new(6);
    pub const SGROUP_ELEMENT: TypeCode = Self::new(7);
    pub const SSIGMAPROP: TypeCode = Self::new(8);
    pub const SANY: TypeCode = Self::new(97);

    pub const COLLECTION_TYPE_CONSTR_ID: u8 = 1;
    pub const COLLECTION_TYPE_CODE: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::COLLECTION_TYPE_CONSTR_ID);

    const fn new(c: u8) -> TypeCode {
        TypeCode(c)
    }

    pub const fn value(&self) -> u8 {
        self.0
    }
}

impl Add for TypeCode {
    type Output = TypeCode;
    fn add(self, rhs: Self) -> TypeCode {
        TypeCode::new(self.0 + rhs.0)
    }
}

impl SigmaSerializable for TypeCode {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u8(self.value())
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        let b = r.get_u8()?;
        match b {
            0 => Err(SerializationError::InvalidTypePrefix),
            _ => Ok(Self(b)),
        }
    }
}

fn get_embeddable_type(code: u8) -> Result<SType, SerializationError> {
    match TypeCode::new(code) {
        TypeCode::SBOOLEAN => Ok(SType::SBoolean),
        TypeCode::SBYTE => Ok(SType::SByte),
        TypeCode::SSHORT => Ok(SType::SShort),
        TypeCode::SINT => Ok(SType::SInt),
        TypeCode::SLONG => Ok(SType::SLong),
        TypeCode::SBIGINT => Ok(SType::SBigInt),
        TypeCode::SGROUP_ELEMENT => Ok(SType::SGroupElement),
        TypeCode::SSIGMAPROP => Ok(SType::SSigmaProp),
        _ => Err(SerializationError::InvalidOpCode),
    }
}

fn is_stype_embeddable(tpe: &SType) -> bool {
    match tpe {
        SType::SBoolean => true,
        SType::SByte => true,
        SType::SShort => true,
        SType::SInt => true,
        SType::SLong => true,
        SType::SBigInt => true,
        SType::SGroupElement => true,
        SType::SSigmaProp => true,
        _ => false,
    }
}

impl SigmaSerializable for SType {
    fn sigma_serialize<W: WriteSigmaVlqExt>(&self, w: &mut W) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        match self {
            SType::SAny => self.type_code().sigma_serialize(w),

            SType::SBoolean => self.type_code().sigma_serialize(w),
            SType::SByte => self.type_code().sigma_serialize(w),
            SType::SShort => self.type_code().sigma_serialize(w),
            SType::SInt => self.type_code().sigma_serialize(w),
            SType::SLong => self.type_code().sigma_serialize(w),
            SType::SBigInt => self.type_code().sigma_serialize(w),
            SType::SGroupElement => self.type_code().sigma_serialize(w),
            SType::SSigmaProp => self.type_code().sigma_serialize(w),

            SType::SBox => todo!(),
            SType::SAvlTree => todo!(),
            SType::SOption(_) => todo!(),
            SType::SColl(elem_type) if is_stype_embeddable(elem_type) => {
                let code = TypeCode::COLLECTION_TYPE_CODE + elem_type.type_code();
                code.sigma_serialize(w)
            }
            SType::SColl(_) => todo!(),
            SType::STup(_) => todo!(),
            SType::SFunc(_) => todo!(),
        }
    }

    fn sigma_parse<R: ReadSigmaVlqExt>(r: &mut R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let type_code = TypeCode::sigma_parse(r)?;
        let constr_id = type_code.value() / TypeCode::PRIM_RANGE;
        let prim_id = type_code.value() % TypeCode::PRIM_RANGE;
        let tpe = match constr_id {
            // primitive
            0 => get_embeddable_type(type_code.value())?,
            // Coll[_]
            1 => {
                let t_elem = get_embeddable_type(prim_id)?;
                SType::SColl(Box::new(t_elem))
            }
            _ => Err(SerializationError::NotImplementedYet("".to_string()))?,
        };
        Ok(tpe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sigma_ser::test_helpers::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<SType>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}

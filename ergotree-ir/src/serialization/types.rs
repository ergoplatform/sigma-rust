#![allow(missing_docs)]

use super::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SerializationError, SigmaSerializable,
};
use crate::types::stuple::STuple;
use crate::types::stype::SType;
use crate::types::stype_param::STypeVar;
use sigma_ser::vlq_encode;
use std::convert::TryInto;
use std::{io, ops::Add};
use vlq_encode::WriteSigmaVlqExt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    pub const SBOX: TypeCode = Self::new(99);
    pub const SAVL_TREE: TypeCode = Self::new(100);
    pub const SCONTEXT: TypeCode = Self::new(101);
    pub const SSTRING: TypeCode = Self::new(102);
    pub const STYPE_VAR: TypeCode = Self::new(103);

    const COLLECTION_CONSTR_ID: u8 = 1;
    pub const COLLECTION: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::COLLECTION_CONSTR_ID);
    const NESTED_COLLECTION_CONSTS_ID: u8 = 2;
    pub const NESTED_COLLECTION: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::NESTED_COLLECTION_CONSTS_ID);

    const TUPLE_PAIR1_CONSTR_ID: u8 = 5;
    pub const TUPLE_PAIR1: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR1_CONSTR_ID);

    const TUPLE_PAIR2_CONSTR_ID: u8 = 6;
    pub const TUPLE_PAIR2: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR2_CONSTR_ID);

    pub const TUPLE_TRIPLE: TypeCode = Self::TUPLE_PAIR2;

    const TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID: u8 = 7;
    pub const TUPLE_PAIR_SYMMETRIC: TypeCode = Self::new(
        (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID,
    );

    pub const TUPLE_QUADRUPLE: TypeCode = Self::TUPLE_PAIR_SYMMETRIC;

    pub const TUPLE: TypeCode = Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * 8);

    const OPTION_CONSTR_ID: u8 = 3;
    const OPTION_COLLECTION_TYPE_CONSTR_ID: u8 = 4;
    pub const OPTION: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::OPTION_CONSTR_ID);
    pub const OPTION_COLLECTION: TypeCode =
        Self::new((TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::OPTION_COLLECTION_TYPE_CONSTR_ID);

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

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
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
        _ => Err(SerializationError::InvalidOpCode(code)),
    }
}

#[allow(clippy::match_like_matches_macro)]
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

/**
 * Each SType is serialized to array of bytes by:
 * - emitting typeCode of each node (see special case for collections below)
 * - then recursively serializing subtrees from left to right on each level
 * - for each collection of primitive type there is special type code to emit single byte instead of two bytes
 * Types code intervals
 * - (1 .. MaxPrimTypeCode)  // primitive types
 * - (CollectionTypeCode .. CollectionTypeCode + MaxPrimTypeCode) // collections of primitive types
 * - (MaxCollectionTypeCode ..)  // Other types
 * Collection of non-primitive type is serialized as (CollectionTypeCode, serialize(elementType))
 */
impl SigmaSerializable for SType {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        match self {
            SType::SFunc(_) => panic!("SFunc is not supposed to be here"),
            SType::SAny => self.type_code().sigma_serialize(w),
            SType::SBoolean => self.type_code().sigma_serialize(w),
            SType::SByte => self.type_code().sigma_serialize(w),
            SType::SShort => self.type_code().sigma_serialize(w),
            SType::SInt => self.type_code().sigma_serialize(w),
            SType::SLong => self.type_code().sigma_serialize(w),
            SType::SBigInt => self.type_code().sigma_serialize(w),
            SType::SGroupElement => self.type_code().sigma_serialize(w),
            SType::SSigmaProp => self.type_code().sigma_serialize(w),
            SType::SBox => self.type_code().sigma_serialize(w),
            SType::SAvlTree => self.type_code().sigma_serialize(w),
            SType::SContext => self.type_code().sigma_serialize(w),
            SType::SOption(elem_type) if is_stype_embeddable(elem_type) => {
                let code = TypeCode::OPTION + elem_type.type_code();
                code.sigma_serialize(w)
            }
            SType::SOption(elem_type) => match &**elem_type {
                SType::SColl(elem_type) if is_stype_embeddable(elem_type.as_ref()) => {
                    let code = TypeCode::OPTION_COLLECTION + elem_type.type_code();
                    code.sigma_serialize(w)
                }
                _ => {
                    TypeCode::OPTION.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },
            SType::SColl(elem_type) if is_stype_embeddable(elem_type) => {
                let code = TypeCode::COLLECTION + elem_type.type_code();
                code.sigma_serialize(w)
            }
            SType::SColl(elem_type) => match &**elem_type {
                SType::SColl(inner_elem_type) if is_stype_embeddable(inner_elem_type.as_ref()) => {
                    let code = TypeCode::NESTED_COLLECTION + inner_elem_type.type_code();
                    code.sigma_serialize(w)
                }
                _ => {
                    TypeCode::COLLECTION.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },
            SType::STuple(STuple { items }) => match items.clone().as_slice() {
                [t1, t2] => match (t1, t2) {
                    (p, _) if is_stype_embeddable(p) => {
                        if p == t2 {
                            // Symmetric pair of primitive types (`(Int, Int)`, `(Byte,Byte)`, etc.)
                            let code = TypeCode::TUPLE_PAIR_SYMMETRIC + p.type_code();
                            code.sigma_serialize(w)
                        } else {
                            // Pair of types where first is primitive (`(_, Int)`)
                            let code = TypeCode::TUPLE_PAIR1 + p.type_code();
                            code.sigma_serialize(w)?;
                            t2.sigma_serialize(w)
                        }
                    }
                    (_, p) if is_stype_embeddable(p) => {
                        // Pair of types where second is primitive (`(Int, _)`)
                        let code = TypeCode::TUPLE_PAIR2 + p.type_code();
                        code.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (_, _) => {
                        // Pair of non-primitive types (`((Int, Byte), (Boolean,Box))`, etc.)
                        TypeCode::TUPLE_PAIR1.sigma_serialize(w)?;
                        t1.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                },
                _ => match items.len() {
                    3 => {
                        TypeCode::TUPLE_TRIPLE.sigma_serialize(w)?;
                        items.iter().try_for_each(|i| i.sigma_serialize(w))
                    }
                    4 => {
                        TypeCode::TUPLE_QUADRUPLE.sigma_serialize(w)?;
                        items.iter().try_for_each(|i| i.sigma_serialize(w))
                    }
                    _ => {
                        TypeCode::TUPLE.sigma_serialize(w)?;
                        w.put_u8(items.len() as u8)?;
                        items.iter().try_for_each(|i| i.sigma_serialize(w))
                    }
                },
            },
            SType::STypeVar(tv) => {
                TypeCode::STYPE_VAR.sigma_serialize(w)?;
                tv.sigma_serialize(w)
            }
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let c = TypeCode::sigma_parse(r)?;

        let tpe = if c.value() < TypeCode::TUPLE.value() {
            let constr_id = c.value() / TypeCode::PRIM_RANGE;
            let prim_id = c.value() % TypeCode::PRIM_RANGE;
            match constr_id {
                // primitive
                0 => get_embeddable_type(c.value())?,
                // Coll[_]
                1 => {
                    let t_elem = get_arg_type(r, prim_id)?;
                    SType::SColl(Box::new(t_elem))
                }
                // Coll[Coll[_]]
                2 => {
                    let t_elem = get_arg_type(r, prim_id)?;
                    SType::SColl(Box::new(SType::SColl(Box::new(t_elem))))
                }
                // Option[_]
                3 => {
                    let t_elem = get_arg_type(r, prim_id)?;
                    SType::SOption(Box::new(t_elem))
                }
                // Option[Coll[_]]
                4 => {
                    let t_elem = get_arg_type(r, prim_id)?;
                    SType::SOption(SType::SColl(t_elem.into()).into())
                }
                TypeCode::TUPLE_PAIR1_CONSTR_ID => {
                    // (_, t2)
                    let (t1, t2) = if prim_id == 0 {
                        // Pair of non-primitive types (`((Int, Byte), (Boolean,Box))`, etc.)
                        (Self::sigma_parse(r)?, Self::sigma_parse(r)?)
                    } else {
                        // Pair of types where first is primitive (`(_, Int)`)
                        (get_embeddable_type(prim_id)?, Self::sigma_parse(r)?)
                    };
                    SType::STuple(vec![t1, t2].try_into().unwrap())
                }
                TypeCode::TUPLE_PAIR2_CONSTR_ID => {
                    // (t1, _)
                    if prim_id == 0 {
                        // Triple of types
                        let t1 = Self::sigma_parse(r)?;
                        let t2 = Self::sigma_parse(r)?;
                        let t3 = Self::sigma_parse(r)?;
                        SType::STuple(vec![t1, t2, t3].try_into().unwrap())
                    } else {
                        // Pair of types where second is primitive (`(Int, _)`)
                        let t2 = get_embeddable_type(prim_id)?;
                        let t1 = Self::sigma_parse(r)?;
                        SType::STuple(vec![t1, t2].try_into().unwrap())
                    }
                }
                TypeCode::TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID => {
                    // (_, _)
                    if prim_id == 0 {
                        // Quadriple of types
                        let t1 = Self::sigma_parse(r)?;
                        let t2 = Self::sigma_parse(r)?;
                        let t3 = Self::sigma_parse(r)?;
                        let t4 = Self::sigma_parse(r)?;
                        SType::STuple(vec![t1, t2, t3, t4].try_into().unwrap())
                    } else {
                        // Symmetric pair of primitive types (`(Int, Int)`, `(Byte,Byte)`, etc.)
                        let t = get_embeddable_type(prim_id)?;
                        SType::STuple(vec![t.clone(), t].try_into().unwrap())
                    }
                }
                _ => {
                    return Err(SerializationError::NotImplementedYet(format!(
                        "case 1: parsing type is not yet implemented(constr_id == {:?})",
                        constr_id
                    )))
                }
            }
        } else {
            match c {
                TypeCode::TUPLE => {
                    let len = r.get_u8()?;
                    let mut items = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        items.push(SType::sigma_parse(r)?);
                    }
                    Ok(SType::STuple(items.try_into().map_err(|_| {
                        SerializationError::TupleItemsOutOfBounds(len as usize)
                    })?))
                }
                TypeCode::SANY => Ok(SType::SAny),
                TypeCode::SBOX => Ok(SType::SBox),
                TypeCode::SAVL_TREE => Ok(SType::SAvlTree),
                TypeCode::SCONTEXT => Ok(SType::SContext),
                TypeCode::STYPE_VAR => Ok(SType::STypeVar(STypeVar::sigma_parse(r)?)),
                _ => Err(SerializationError::NotImplementedYet(format!(
                    "case 2: parsing type is not yet implemented(c == {:?})",
                    c
                ))),
            }?
        };
        Ok(tpe)
    }
}

fn get_arg_type<R: SigmaByteRead>(r: &mut R, prim_id: u8) -> Result<SType, SerializationError> {
    if prim_id == 0 {
        SType::sigma_parse(r)
    } else {
        get_embeddable_type(prim_id)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<SType>()) {
            dbg!(&v);
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}

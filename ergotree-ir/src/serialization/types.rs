use super::op_code::OpCode;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SigmaSerializationError;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::types::stuple;
use crate::types::stype::SType;
use crate::types::stype_param;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::TryInto;

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)] // to differentiate from similarly named SType enum variants
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum TypeCode {
    SBOOLEAN = 1,
    SBYTE = 2,
    SSHORT = 3,
    SINT = 4,
    SLONG = 5,
    SBIGINT = 6,
    SGROUP_ELEMENT = 7,
    SSIGMAPROP = 8,

    COLL = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::COLLECTION_CONSTR_ID, // 12 * 1
    COLL_BOOL = TypeCode::COLL as u8 + TypeCode::SBOOLEAN as u8,               // 13
    COLL_BYTE = TypeCode::COLL as u8 + TypeCode::SBYTE as u8,                  // 14
    COLL_SHORT = TypeCode::COLL as u8 + TypeCode::SSHORT as u8,                // 15
    COLL_INT = TypeCode::COLL as u8 + TypeCode::SINT as u8,                    // 16
    COLL_LONG = TypeCode::COLL as u8 + TypeCode::SLONG as u8,                  // 17
    COLL_BIGINT = TypeCode::COLL as u8 + TypeCode::SBIGINT as u8,              // 18
    COLL_GROUP_ELEMENT = TypeCode::COLL as u8 + TypeCode::SGROUP_ELEMENT as u8, // 19
    COLL_SIGMAPROP = TypeCode::COLL as u8 + TypeCode::SSIGMAPROP as u8,        // 20

    NESTED_COLL_BOOL = TypeCode::NESTED_COLL + TypeCode::SBOOLEAN as u8, // 25
    NESTED_COLL_BYTE = TypeCode::NESTED_COLL + TypeCode::SBYTE as u8,    // 26
    NESTED_COLL_SHORT = TypeCode::NESTED_COLL + TypeCode::SSHORT as u8,  // 27
    NESTED_COLL_INT = TypeCode::NESTED_COLL + TypeCode::SINT as u8,      // 28
    NESTED_COLL_LONG = TypeCode::NESTED_COLL + TypeCode::SLONG as u8,    // 29
    NESTED_COLL_BIGINT = TypeCode::NESTED_COLL + TypeCode::SBIGINT as u8, // 30
    NESTED_COLL_GROUP_ELEMENT = TypeCode::NESTED_COLL + TypeCode::SGROUP_ELEMENT as u8, // 31
    NESTED_COLL_SIGMAPROP = TypeCode::NESTED_COLL + TypeCode::SSIGMAPROP as u8, // 32

    OPTION = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::OPTION_CONSTR_ID, // 12 * 3 = 36
    OPTION_BOOL = TypeCode::OPTION as u8 + TypeCode::SBOOLEAN as u8,         // 37
    OPTION_BYTE = TypeCode::OPTION as u8 + TypeCode::SBYTE as u8,            // 38
    OPTION_SHORT = TypeCode::OPTION as u8 + TypeCode::SSHORT as u8,          // 39
    OPTION_INT = TypeCode::OPTION as u8 + TypeCode::SINT as u8,              // 40
    OPTION_LONG = TypeCode::OPTION as u8 + TypeCode::SLONG as u8,            // 41
    OPTION_BIGINT = TypeCode::OPTION as u8 + TypeCode::SBIGINT as u8,        // 42
    OPTION_GROUP_ELEMENT = TypeCode::OPTION as u8 + TypeCode::SGROUP_ELEMENT as u8, // 43
    OPTION_SIGMAPROP = TypeCode::OPTION as u8 + TypeCode::SSIGMAPROP as u8,  // 44

    OPTION_COLL_BOOL = TypeCode::OPTION_COLLECTION + TypeCode::SBOOLEAN as u8, // 49
    OPTION_COLL_BYTE = TypeCode::OPTION_COLLECTION + TypeCode::SBYTE as u8,    // 50
    OPTION_COLL_SHORT = TypeCode::OPTION_COLLECTION + TypeCode::SSHORT as u8,  // 51
    OPTION_COLL_INT = TypeCode::OPTION_COLLECTION + TypeCode::SINT as u8,      // 52
    OPTION_COLL_LONG = TypeCode::OPTION_COLLECTION + TypeCode::SLONG as u8,    // 53
    OPTION_COLL_BIGINT = TypeCode::OPTION_COLLECTION + TypeCode::SBIGINT as u8, // 54
    OPTION_COLL_GROUP_ELEMENT = TypeCode::OPTION_COLLECTION + TypeCode::SGROUP_ELEMENT as u8, // 55
    OPTION_COLL_SIGMAPROP = TypeCode::OPTION_COLLECTION + TypeCode::SSIGMAPROP as u8, // 56

    TUPLE_PAIR1 = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR1_CONSTR_ID, // 12 * 5 = 60
    TUPLE_PAIR1_BOOL = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SBOOLEAN as u8,         // 61
    TUPLE_PAIR1_BYTE = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SBYTE as u8,            // 62
    TUPLE_PAIR1_SHORT = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SSHORT as u8,          // 63
    TUPLE_PAIR1_INT = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SINT as u8,              // 64
    TUPLE_PAIR1_LONG = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SLONG as u8,            // 65
    TUPLE_PAIR1_BIGINT = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SBIGINT as u8,        // 66
    TUPLE_PAIR1_GROUP_ELEMENT = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SGROUP_ELEMENT as u8, // 67
    TUPLE_PAIR1_SIGMAPROP = TypeCode::TUPLE_PAIR1 as u8 + TypeCode::SSIGMAPROP as u8,  // 68

    TUPLE_TRIPLE = Self::TUPLE_PAIR2, // 72

    TUPLE_PAIR2_BOOL = TypeCode::TUPLE_PAIR2 + TypeCode::SBOOLEAN as u8, // 73
    TUPLE_PAIR2_BYTE = TypeCode::TUPLE_PAIR2 + TypeCode::SBYTE as u8,    // 74
    TUPLE_PAIR2_SHORT = TypeCode::TUPLE_PAIR2 + TypeCode::SSHORT as u8,  // 75
    TUPLE_PAIR2_INT = TypeCode::TUPLE_PAIR2 + TypeCode::SINT as u8,      // 76
    TUPLE_PAIR2_LONG = TypeCode::TUPLE_PAIR2 + TypeCode::SLONG as u8,    // 77
    TUPLE_PAIR2_BIGINT = TypeCode::TUPLE_PAIR2 + TypeCode::SBIGINT as u8, // 78
    TUPLE_PAIR2_GROUP_ELEMENT = TypeCode::TUPLE_PAIR2 + TypeCode::SGROUP_ELEMENT as u8, // 79
    TUPLE_PAIR2_SIGMAPROP = TypeCode::TUPLE_PAIR2 + TypeCode::SSIGMAPROP as u8, // 80

    TUPLE_QUADRUPLE = Self::TUPLE_PAIR_SYMMETRIC, // 84

    TUPLE_PAIR_SYMMETRIC_BOOL = TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SBOOLEAN as u8, // 85
    TUPLE_PAIR_SYMMETRIC_BYTE = TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SBYTE as u8, // 86
    TUPLE_PAIR_SYMMETRIC_SHORT = TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SSHORT as u8, // 87
    TUPLE_PAIR_SYMMETRIC_INT = TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SINT as u8, // 88
    TUPLE_PAIR_SYMMETRIC_LONG = TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SLONG as u8, // 89
    TUPLE_PAIR_SYMMETRIC_BIGINT = TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SBIGINT as u8, // 90
    TUPLE_PAIR_SYMMETRIC_GROUP_ELEMENT =
        TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SGROUP_ELEMENT as u8, // 91
    TUPLE_PAIR_SYMMETRIC_SIGMAPROP =
        TypeCode::TUPLE_PAIR_SYMMETRIC + TypeCode::SSIGMAPROP as u8, // 92

    TUPLE = (TypeCode::MAX_PRIM_TYPECODE + 1) * 8, // 12 * 8 = 96

    SANY = 97,
    SUNIT = 98,
    SBOX = 99,
    SAVL_TREE = 100,
    SCONTEXT = 101,
    // SSTRING = 102,
    STYPE_VAR = 103,
    SHEADER = 104,
    SPRE_HEADER = 105,
    SGLOBAL = 106,
}

impl TypeCode {
    /// SFunc types occupy remaining space of byte values [FirstFuncType .. 255]
    #[allow(dead_code)]
    const FIRST_FUNC_TYPE: u8 = OpCode::LAST_DATA_TYPE.value();
    #[allow(dead_code)]
    const LAST_FUNC_TYPE: u8 = 255;

    /// Type code of the last valid prim type so that (1 to LastPrimTypeCode) is a range of valid codes.
    #[allow(dead_code)]
    const LAST_PRIM_TYPECODE: u8 = 8;

    /// Upper limit of the interval of valid type codes for primitive types
    const MAX_PRIM_TYPECODE: u8 = 11;

    const COLLECTION_CONSTR_ID: u8 = 1;

    const NESTED_COLLECTION_CONSTS_ID: u8 = 2;
    const NESTED_COLL: u8 =
        (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::NESTED_COLLECTION_CONSTS_ID; // 12 * 2 = 24

    const OPTION_CONSTR_ID: u8 = 3;
    const OPTION_COLLECTION_TYPE_CONSTR_ID: u8 = 4;
    const OPTION_COLLECTION: u8 =
        (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::OPTION_COLLECTION_TYPE_CONSTR_ID; // 12 * 4 = 48

    const TUPLE_PAIR1_CONSTR_ID: u8 = 5;

    const TUPLE_PAIR2_CONSTR_ID: u8 = 6;
    const TUPLE_PAIR2: u8 = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR2_CONSTR_ID; // 12 * 6 = 72

    const TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID: u8 = 7;
    const TUPLE_PAIR_SYMMETRIC: u8 =
        (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID; // 12 * 7 = 84

    /// Parse type code from byte
    pub(crate) fn parse(b: u8) -> Result<Self, SigmaParsingError> {
        match FromPrimitive::from_u8(b) {
            Some(t) => Ok(t),
            None => Err(SigmaParsingError::InvalidTypeCode(b)),
        }
    }

    pub(crate) const fn value(&self) -> u8 {
        *self as u8
    }
}

impl SigmaSerializable for TypeCode {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.value())?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        let b = r.get_u8()?;
        Self::parse(b)
    }
}

impl SType {
    /// Parse type from byte stream. This function should be used instead of
    /// `sigma_parse` when type code is already read for look-ahead
    pub(crate) fn parse_with_type_code<R: SigmaByteRead>(
        r: &mut R,
        c: TypeCode,
    ) -> Result<Self, SigmaParsingError> {
        use SType::*;
        Ok(match c {
            TypeCode::SBOOLEAN => SBoolean,
            TypeCode::SBYTE => SByte,
            TypeCode::SSHORT => SShort,
            TypeCode::SINT => SInt,
            TypeCode::SLONG => SLong,
            TypeCode::SBIGINT => SBigInt,
            TypeCode::SGROUP_ELEMENT => SGroupElement,
            TypeCode::SSIGMAPROP => SSigmaProp,

            TypeCode::COLL => SColl(SType::sigma_parse(r)?.into()),
            TypeCode::COLL_BOOL => SColl(SBoolean.into()),
            TypeCode::COLL_BYTE => SColl(SByte.into()),
            TypeCode::COLL_SHORT => SColl(SShort.into()),
            TypeCode::COLL_INT => SColl(SInt.into()),
            TypeCode::COLL_LONG => SColl(SLong.into()),
            TypeCode::COLL_BIGINT => SColl(SBigInt.into()),
            TypeCode::COLL_GROUP_ELEMENT => SColl(SGroupElement.into()),
            TypeCode::COLL_SIGMAPROP => SColl(SSigmaProp.into()),

            TypeCode::NESTED_COLL_BOOL => SColl(SColl(SBoolean.into()).into()),
            TypeCode::NESTED_COLL_BYTE => SColl(SColl(SByte.into()).into()),
            TypeCode::NESTED_COLL_SHORT => SColl(SColl(SShort.into()).into()),
            TypeCode::NESTED_COLL_INT => SColl(SColl(SInt.into()).into()),
            TypeCode::NESTED_COLL_LONG => SColl(SColl(SLong.into()).into()),
            TypeCode::NESTED_COLL_BIGINT => SColl(SColl(SBigInt.into()).into()),
            TypeCode::NESTED_COLL_GROUP_ELEMENT => SColl(SColl(SGroupElement.into()).into()),
            TypeCode::NESTED_COLL_SIGMAPROP => SColl(SColl(SSigmaProp.into()).into()),

            TypeCode::OPTION => SOption(SType::sigma_parse(r)?.into()),
            TypeCode::OPTION_BOOL => SOption(SBoolean.into()),
            TypeCode::OPTION_BYTE => SOption(SByte.into()),
            TypeCode::OPTION_SHORT => SOption(SShort.into()),
            TypeCode::OPTION_INT => SOption(SInt.into()),
            TypeCode::OPTION_LONG => SOption(SLong.into()),
            TypeCode::OPTION_BIGINT => SOption(SBigInt.into()),
            TypeCode::OPTION_GROUP_ELEMENT => SOption(SGroupElement.into()),
            TypeCode::OPTION_SIGMAPROP => SOption(SSigmaProp.into()),

            TypeCode::OPTION_COLL_BOOL => SOption(SColl(SBoolean.into()).into()),
            TypeCode::OPTION_COLL_BYTE => SOption(SColl(SByte.into()).into()),
            TypeCode::OPTION_COLL_SHORT => SOption(SColl(SShort.into()).into()),
            TypeCode::OPTION_COLL_INT => SOption(SColl(SInt.into()).into()),
            TypeCode::OPTION_COLL_LONG => SOption(SColl(SLong.into()).into()),
            TypeCode::OPTION_COLL_BIGINT => SOption(SColl(SBigInt.into()).into()),
            TypeCode::OPTION_COLL_GROUP_ELEMENT => SOption(SColl(SGroupElement.into()).into()),
            TypeCode::OPTION_COLL_SIGMAPROP => SOption(SColl(SSigmaProp.into()).into()),

            TypeCode::TUPLE_PAIR1 => STuple(stuple::STuple::pair(
                SType::sigma_parse(r)?,
                SType::sigma_parse(r)?,
            )),
            TypeCode::TUPLE_PAIR1_BOOL => {
                STuple(stuple::STuple::pair(SBoolean, SType::sigma_parse(r)?))
            }
            TypeCode::TUPLE_PAIR1_BYTE => {
                STuple(stuple::STuple::pair(SByte, SType::sigma_parse(r)?))
            }
            TypeCode::TUPLE_PAIR1_SHORT => {
                STuple(stuple::STuple::pair(SShort, SType::sigma_parse(r)?))
            }
            TypeCode::TUPLE_PAIR1_INT => STuple(stuple::STuple::pair(SInt, SType::sigma_parse(r)?)),
            TypeCode::TUPLE_PAIR1_LONG => {
                STuple(stuple::STuple::pair(SLong, SType::sigma_parse(r)?))
            }
            TypeCode::TUPLE_PAIR1_BIGINT => {
                STuple(stuple::STuple::pair(SBigInt, SType::sigma_parse(r)?))
            }
            TypeCode::TUPLE_PAIR1_GROUP_ELEMENT => {
                STuple(stuple::STuple::pair(SGroupElement, SType::sigma_parse(r)?))
            }
            TypeCode::TUPLE_PAIR1_SIGMAPROP => {
                STuple(stuple::STuple::pair(SSigmaProp, SType::sigma_parse(r)?))
            }

            TypeCode::TUPLE_TRIPLE => STuple(stuple::STuple::triple(
                SType::sigma_parse(r)?,
                SType::sigma_parse(r)?,
                SType::sigma_parse(r)?,
            )),

            TypeCode::TUPLE_PAIR2_BOOL => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SBoolean))
            }
            TypeCode::TUPLE_PAIR2_BYTE => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SByte))
            }
            TypeCode::TUPLE_PAIR2_SHORT => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SShort))
            }
            TypeCode::TUPLE_PAIR2_INT => STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SInt)),
            TypeCode::TUPLE_PAIR2_LONG => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SLong))
            }
            TypeCode::TUPLE_PAIR2_BIGINT => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SBigInt))
            }
            TypeCode::TUPLE_PAIR2_GROUP_ELEMENT => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SGroupElement))
            }
            TypeCode::TUPLE_PAIR2_SIGMAPROP => {
                STuple(stuple::STuple::pair(SType::sigma_parse(r)?, SSigmaProp))
            }

            TypeCode::TUPLE_QUADRUPLE => STuple(stuple::STuple::quadruple(
                SType::sigma_parse(r)?,
                SType::sigma_parse(r)?,
                SType::sigma_parse(r)?,
                SType::sigma_parse(r)?,
            )),

            TypeCode::TUPLE_PAIR_SYMMETRIC_BOOL => STuple(stuple::STuple::pair(SBoolean, SBoolean)),
            TypeCode::TUPLE_PAIR_SYMMETRIC_BYTE => STuple(stuple::STuple::pair(SByte, SByte)),
            TypeCode::TUPLE_PAIR_SYMMETRIC_SHORT => STuple(stuple::STuple::pair(SShort, SShort)),
            TypeCode::TUPLE_PAIR_SYMMETRIC_INT => STuple(stuple::STuple::pair(SInt, SInt)),
            TypeCode::TUPLE_PAIR_SYMMETRIC_LONG => STuple(stuple::STuple::pair(SLong, SLong)),
            TypeCode::TUPLE_PAIR_SYMMETRIC_BIGINT => STuple(stuple::STuple::pair(SBigInt, SBigInt)),
            TypeCode::TUPLE_PAIR_SYMMETRIC_GROUP_ELEMENT => {
                STuple(stuple::STuple::pair(SGroupElement, SGroupElement))
            }
            TypeCode::TUPLE_PAIR_SYMMETRIC_SIGMAPROP => {
                STuple(stuple::STuple::pair(SSigmaProp, SSigmaProp))
            }

            TypeCode::TUPLE => {
                let len = r.get_u8()?;
                let mut items = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    items.push(SType::sigma_parse(r)?);
                }
                SType::STuple(
                    items
                        .try_into()
                        .map_err(|_| SigmaParsingError::TupleItemsOutOfBounds(len as usize))?,
                )
            }

            TypeCode::SANY => SAny,
            TypeCode::SUNIT => SUnit,
            TypeCode::SBOX => SBox,
            TypeCode::SAVL_TREE => SAvlTree,
            TypeCode::SCONTEXT => SContext,
            TypeCode::STYPE_VAR => STypeVar(stype_param::STypeVar::sigma_parse(r)?),
            TypeCode::SHEADER => SHeader,
            TypeCode::SPRE_HEADER => SPreHeader,
            TypeCode::SGLOBAL => SGlobal,
        })
    }
}

/// Each SType is serialized to array of bytes by:
/// - emitting typeCode of each node (see special case for collections below)
/// - then recursively serializing subtrees from left to right on each level
/// - for each collection of primitive type there is special type code to emit single byte instead of two bytes
/// Types code intervals
/// - (1 .. MaxPrimTypeCode)  // primitive types
/// - (CollectionTypeCode .. CollectionTypeCode + MaxPrimTypeCode) // collections of primitive types
/// - (MaxCollectionTypeCode ..)  // Other types
/// Collection of non-primitive type is serialized as (CollectionTypeCode, serialize(elementType))
impl SigmaSerializable for SType {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        use SType::*;
        match self {
            SType::SFunc(_) => Err(SigmaSerializationError::NotSupported("SFunc")),
            SType::SAny => TypeCode::SANY.sigma_serialize(w),
            SType::SUnit => TypeCode::SUNIT.sigma_serialize(w),
            SType::SBoolean => TypeCode::SBOOLEAN.sigma_serialize(w),
            SType::SByte => TypeCode::SBYTE.sigma_serialize(w),
            SType::SShort => TypeCode::SSHORT.sigma_serialize(w),
            SType::SInt => TypeCode::SINT.sigma_serialize(w),
            SType::SLong => TypeCode::SLONG.sigma_serialize(w),
            SType::SBigInt => TypeCode::SBIGINT.sigma_serialize(w),
            SType::SGroupElement => TypeCode::SGROUP_ELEMENT.sigma_serialize(w),
            SType::SSigmaProp => TypeCode::SSIGMAPROP.sigma_serialize(w),
            SType::SBox => TypeCode::SBOX.sigma_serialize(w),
            SType::SAvlTree => TypeCode::SAVL_TREE.sigma_serialize(w),
            SType::SContext => TypeCode::SCONTEXT.sigma_serialize(w),
            SType::SHeader => TypeCode::SHEADER.sigma_serialize(w),
            SType::SPreHeader => TypeCode::SPRE_HEADER.sigma_serialize(w),
            SType::SGlobal => TypeCode::SGLOBAL.sigma_serialize(w),
            SOption(elem_type) => match &**elem_type {
                SBoolean => TypeCode::OPTION_BOOL.sigma_serialize(w),
                SByte => TypeCode::OPTION_BYTE.sigma_serialize(w),
                SShort => TypeCode::OPTION_SHORT.sigma_serialize(w),
                SInt => TypeCode::OPTION_INT.sigma_serialize(w),
                SLong => TypeCode::OPTION_LONG.sigma_serialize(w),
                SBigInt => TypeCode::OPTION_BIGINT.sigma_serialize(w),
                SGroupElement => TypeCode::OPTION_GROUP_ELEMENT.sigma_serialize(w),
                SSigmaProp => TypeCode::OPTION_SIGMAPROP.sigma_serialize(w),
                SColl(inner_elem_type) => match &**inner_elem_type {
                    SBoolean => TypeCode::OPTION_COLL_BOOL.sigma_serialize(w),
                    SByte => TypeCode::OPTION_COLL_BYTE.sigma_serialize(w),
                    SShort => TypeCode::OPTION_COLL_SHORT.sigma_serialize(w),
                    SInt => TypeCode::OPTION_COLL_INT.sigma_serialize(w),
                    SLong => TypeCode::OPTION_COLL_LONG.sigma_serialize(w),
                    SBigInt => TypeCode::OPTION_COLL_BIGINT.sigma_serialize(w),
                    SGroupElement => TypeCode::OPTION_COLL_GROUP_ELEMENT.sigma_serialize(w),
                    SSigmaProp => TypeCode::OPTION_COLL_SIGMAPROP.sigma_serialize(w),
                    STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                    | STuple(_) | SFunc(_) | SContext | SHeader | SPreHeader | SGlobal => {
                        // if not "embeddable" type fallback to generic Option type code following
                        // elem type code
                        TypeCode::OPTION.sigma_serialize(w)?;
                        elem_type.sigma_serialize(w)
                    }
                },
                STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | STuple(_)
                | SFunc(_) | SContext | SHeader | SPreHeader | SGlobal => {
                    // if not "embeddable" type fallback to generic Option type code following
                    // elem type code
                    TypeCode::OPTION.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },

            SType::SColl(elem_type) => match &**elem_type {
                SBoolean => TypeCode::COLL_BOOL.sigma_serialize(w),
                SByte => TypeCode::COLL_BYTE.sigma_serialize(w),
                SShort => TypeCode::COLL_SHORT.sigma_serialize(w),
                SInt => TypeCode::COLL_INT.sigma_serialize(w),
                SLong => TypeCode::COLL_LONG.sigma_serialize(w),
                SBigInt => TypeCode::COLL_BIGINT.sigma_serialize(w),
                SGroupElement => TypeCode::COLL_GROUP_ELEMENT.sigma_serialize(w),
                SSigmaProp => TypeCode::COLL_SIGMAPROP.sigma_serialize(w),
                SColl(inner_elem_type) => match &**inner_elem_type {
                    SBoolean => TypeCode::NESTED_COLL_BOOL.sigma_serialize(w),
                    SByte => TypeCode::NESTED_COLL_BYTE.sigma_serialize(w),
                    SShort => TypeCode::NESTED_COLL_SHORT.sigma_serialize(w),
                    SInt => TypeCode::NESTED_COLL_INT.sigma_serialize(w),
                    SLong => TypeCode::NESTED_COLL_LONG.sigma_serialize(w),
                    SBigInt => TypeCode::NESTED_COLL_BIGINT.sigma_serialize(w),
                    SGroupElement => TypeCode::NESTED_COLL_GROUP_ELEMENT.sigma_serialize(w),
                    SSigmaProp => TypeCode::NESTED_COLL_SIGMAPROP.sigma_serialize(w),
                    STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                    | STuple(_) | SFunc(_) | SContext | SHeader | SPreHeader | SGlobal => {
                        // if not "embeddable" type fallback to generic Coll type code following
                        // elem type code
                        TypeCode::COLL.sigma_serialize(w)?;
                        elem_type.sigma_serialize(w)
                    }
                },
                STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | STuple(_)
                | SFunc(_) | SContext | SHeader | SPreHeader | SGlobal => {
                    // if not "embeddable" type fallback to generic Coll type code following
                    // elem type code
                    TypeCode::COLL.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },
            SType::STuple(stuple::STuple { items }) => match items.clone().as_slice() {
                [t1, t2] => match (t1, t2) {
                    (SBoolean, SBoolean) => TypeCode::TUPLE_PAIR_SYMMETRIC_BOOL.sigma_serialize(w),
                    (SByte, SByte) => TypeCode::TUPLE_PAIR_SYMMETRIC_BYTE.sigma_serialize(w),
                    (SInt, SInt) => TypeCode::TUPLE_PAIR_SYMMETRIC_INT.sigma_serialize(w),
                    (SLong, SLong) => TypeCode::TUPLE_PAIR_SYMMETRIC_LONG.sigma_serialize(w),
                    (SBigInt, SBigInt) => TypeCode::TUPLE_PAIR_SYMMETRIC_BIGINT.sigma_serialize(w),
                    (SGroupElement, SGroupElement) => {
                        TypeCode::TUPLE_PAIR_SYMMETRIC_GROUP_ELEMENT.sigma_serialize(w)
                    }
                    (SSigmaProp, SSigmaProp) => {
                        TypeCode::TUPLE_PAIR_SYMMETRIC_SIGMAPROP.sigma_serialize(w)
                    }

                    (SBoolean, t2) => {
                        TypeCode::TUPLE_PAIR1_BOOL.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SByte, t2) => {
                        TypeCode::TUPLE_PAIR1_BYTE.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SShort, t2) => {
                        TypeCode::TUPLE_PAIR1_SHORT.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SInt, t2) => {
                        TypeCode::TUPLE_PAIR1_INT.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SLong, t2) => {
                        TypeCode::TUPLE_PAIR1_LONG.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SBigInt, t2) => {
                        TypeCode::TUPLE_PAIR1_BIGINT.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SGroupElement, t2) => {
                        TypeCode::TUPLE_PAIR1_GROUP_ELEMENT.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                    (SSigmaProp, t2) => {
                        TypeCode::TUPLE_PAIR1_SIGMAPROP.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }

                    (t1, SBoolean) => {
                        TypeCode::TUPLE_PAIR2_BOOL.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SByte) => {
                        TypeCode::TUPLE_PAIR2_BYTE.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SShort) => {
                        TypeCode::TUPLE_PAIR2_SHORT.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SInt) => {
                        TypeCode::TUPLE_PAIR2_INT.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SLong) => {
                        TypeCode::TUPLE_PAIR2_LONG.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SBigInt) => {
                        TypeCode::TUPLE_PAIR2_BIGINT.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SGroupElement) => {
                        TypeCode::TUPLE_PAIR2_GROUP_ELEMENT.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (t1, SSigmaProp) => {
                        TypeCode::TUPLE_PAIR2_SIGMAPROP.sigma_serialize(w)?;
                        t1.sigma_serialize(w)
                    }
                    (
                        STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                        | STuple(_) | SFunc(_) | SContext | SHeader | SPreHeader | SGlobal,
                        STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                        | STuple(_) | SFunc(_) | SContext | SHeader | SPreHeader | SGlobal,
                    ) => {
                        // Pair of non-primitive types (`(SBox, SAvlTree)`, `((Int, Byte), (Boolean,Box))`, etc.)
                        TypeCode::TUPLE_PAIR1.sigma_serialize(w)?;
                        t1.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                },
                [t1, t2, t3] => {
                    TypeCode::TUPLE_TRIPLE.sigma_serialize(w)?;
                    t1.sigma_serialize(w)?;
                    t2.sigma_serialize(w)?;
                    t3.sigma_serialize(w)
                }
                [t1, t2, t3, t4] => {
                    TypeCode::TUPLE_QUADRUPLE.sigma_serialize(w)?;
                    t1.sigma_serialize(w)?;
                    t2.sigma_serialize(w)?;
                    t3.sigma_serialize(w)?;
                    t4.sigma_serialize(w)
                }
                _ => {
                    TypeCode::TUPLE.sigma_serialize(w)?;
                    w.put_u8(items.len() as u8)?;
                    items.iter().try_for_each(|i| i.sigma_serialize(w))
                }
            },

            SType::STypeVar(tv) => {
                TypeCode::STYPE_VAR.sigma_serialize(w)?;
                tv.sigma_serialize(w)
            }
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let c = TypeCode::sigma_parse(r)?;
        Self::parse_with_type_code(r, c)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<SType>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}

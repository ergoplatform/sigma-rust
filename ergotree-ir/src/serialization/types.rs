use super::op_code::OpCode;
use super::sigma_byte_writer::SigmaByteWrite;
use super::SigmaSerializationError;
use crate::ergo_tree::ErgoTreeVersion;
use crate::serialization::SigmaSerializeResult;
use crate::serialization::{
    sigma_byte_reader::SigmaByteRead, SigmaParsingError, SigmaSerializable,
};
use crate::types::stuple;
use crate::types::stype::SType;
use crate::types::stype_param::STypeParam;
use alloc::vec::Vec;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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
    SUNSIGNEDBIGINT = 9,
    COLL = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::COLLECTION_CONSTR_ID, // 12 * 1
    NESTED_COLL = (TypeCode::MAX_PRIM_TYPECODE + 1) * (2 * TypeCode::COLLECTION_CONSTR_ID), // 12 * 2 = 24
    OPTION = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::OPTION_CONSTR_ID, // 12 * 3 = 36
    OPTION_COLL = (TypeCode::MAX_PRIM_TYPECODE + 1)
        * (TypeCode::COLLECTION_CONSTR_ID + TypeCode::OPTION_CONSTR_ID), // 12 * 4 = 48

    TUPLE_PAIR1 = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR1_CONSTR_ID, // 12 * 5 = 60
    TUPLE_PAIR2 = (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR2_CONSTR_ID, // 72
    TUPLE_PAIR_SYMMETRIC =
        (TypeCode::MAX_PRIM_TYPECODE + 1) * TypeCode::TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID, // 84
    TUPLE = (TypeCode::MAX_PRIM_TYPECODE + 1) * 8, // 12 * 8 = 96
    SANY = 97,
    SUNIT = 98,
    SBOX = 99,
    SAVL_TREE = 100,
    SCONTEXT = 101,
    SSTRING = 102,
    STYPE_VAR = 103,
    SHEADER = 104,
    SPRE_HEADER = 105,
    SGLOBAL = 106,
    SFUNC = 112,
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
    const TUPLE_TYPECODE: u8 = (TypeCode::MAX_PRIM_TYPECODE + 1) * 8; // 12 * 8 = 96

    const COLLECTION_CONSTR_ID: u8 = 1;

    const OPTION_CONSTR_ID: u8 = 3;

    const TUPLE_PAIR1_CONSTR_ID: u8 = 5;

    const TUPLE_PAIR2_CONSTR_ID: u8 = 6;

    const TUPLE_PAIR_SYMMETRIC_TYPE_CONSTR_ID: u8 = 7;

    /// Parse type code from byte
    fn parse(b: u8) -> Result<Self, SigmaParsingError> {
        match FromPrimitive::from_u8(b) {
            Some(t) => Ok(t),
            None => Err(SigmaParsingError::InvalidTypeCode(b)),
        }
    }

    /// Unpack serialized tag byte. Returns Ok((container, embeddable_type)) if parsing is successful. If embeddable_type is None then need to read-ahead in byte-stream to obtain type
    fn unpack_tag(tag: u8) -> Result<(Option<Self>, Option<Self>), SigmaParsingError> {
        if tag < TypeCode::TUPLE_TYPECODE {
            let container_id =
                (tag / (TypeCode::MAX_PRIM_TYPECODE + 1)) * (TypeCode::MAX_PRIM_TYPECODE + 1);
            let type_id = tag % (TypeCode::MAX_PRIM_TYPECODE + 1);
            let container_code = if container_id == 0 {
                None
            } else {
                Some(TypeCode::parse(container_id)?)
            };
            let type_code = if type_id == 0 {
                None
            } else {
                Some(TypeCode::parse(type_id)?)
            };
            Ok((container_code, type_code))
        } else {
            Ok((None, None))
        }
    }

    fn get_embeddable_type(
        &self,
        tree_version: ErgoTreeVersion,
    ) -> Result<SType, SigmaParsingError> {
        use SType::*;
        // TODO: UnsignedBigInt
        match self {
            TypeCode::SBOOLEAN => Ok(SBoolean),
            TypeCode::SBYTE => Ok(SByte),
            TypeCode::SSHORT => Ok(SShort),
            TypeCode::SINT => Ok(SInt),
            TypeCode::SLONG => Ok(SLong),
            TypeCode::SBIGINT => Ok(SBigInt),
            TypeCode::SGROUP_ELEMENT => Ok(SGroupElement),
            TypeCode::SSIGMAPROP => Ok(SSigmaProp),
            TypeCode::SUNSIGNEDBIGINT if tree_version >= ErgoTreeVersion::V3 => Ok(SUnsignedBigInt),
            _ => Err(SigmaParsingError::InvalidTypeCode(*self as u8)),
        }
    }

    fn from_primitive_type(stype: &SType) -> Option<Self> {
        Some(match stype {
            SType::SAny => TypeCode::SANY,
            SType::SUnit => TypeCode::SUNIT,
            SType::SBoolean => TypeCode::SBOOLEAN,
            SType::SByte => TypeCode::SBYTE,
            SType::SShort => TypeCode::SSHORT,
            SType::SInt => TypeCode::SINT,
            SType::SLong => TypeCode::SLONG,
            SType::SBigInt => TypeCode::SBIGINT,
            SType::SGroupElement => TypeCode::SGROUP_ELEMENT,
            SType::SSigmaProp => TypeCode::SSIGMAPROP,
            SType::SUnsignedBigInt => TypeCode::SUNSIGNEDBIGINT,
            SType::SBox => TypeCode::SBOX,
            SType::SAvlTree => TypeCode::SAVL_TREE,
            SType::SContext => TypeCode::SCONTEXT,
            SType::SString => TypeCode::SSTRING,
            SType::SHeader => TypeCode::SHEADER,
            SType::SPreHeader => TypeCode::SPRE_HEADER,
            SType::SGlobal => TypeCode::SGLOBAL,
            SType::STypeVar(_)
            | SType::SOption(_)
            | SType::SColl(_)
            | SType::STuple(_)
            | SType::SFunc(_) => return None,
        })
    }

    pub(crate) const fn value(&self) -> u8 {
        *self as u8
    }
}

impl SigmaSerializable for TypeCode {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        w.put_u8(self.value())?;
        // Each serialized type code is one byte (PutByteCost) -- charged for Constant type
        // prefixes (e.g. box register types) during Global.serialize; gated, no-op otherwise.
        w.add_put_byte_cost();
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
    pub(crate) fn parse_with_tag<R: SigmaByteRead>(
        r: &mut R,
        c: u8,
    ) -> Result<Self, SigmaParsingError> {
        use SType::*;
        if c < TypeCode::TUPLE_TYPECODE {
            let (container, embeddable) = TypeCode::unpack_tag(c)?;
            let mut stype = || {
                embeddable
                    .map(|e| e.get_embeddable_type(r.tree_version()))
                    .unwrap_or_else(|| SType::sigma_parse(r))
            };
            Ok(match container {
                None => {
                    if let Some(embeddable) = embeddable {
                        embeddable.get_embeddable_type(r.tree_version())?
                    } else {
                        return Err(SigmaParsingError::InvalidTypeCode(c));
                    }
                }
                Some(TypeCode::COLL) => SColl(stype()?.into()),
                Some(TypeCode::NESTED_COLL) => SColl(SColl(stype()?.into()).into()),
                Some(TypeCode::OPTION) => SOption(stype()?.into()),
                Some(TypeCode::OPTION_COLL) => SOption(SColl(stype()?.into()).into()),
                Some(TypeCode::TUPLE_PAIR1) => {
                    STuple(stuple::STuple::pair(stype()?, SType::sigma_parse(r)?))
                }
                Some(TypeCode::TUPLE_PAIR2) if embeddable.is_none() => {
                    STuple(stuple::STuple::triple(
                        stype()?,
                        SType::sigma_parse(r)?,
                        SType::sigma_parse(r)?,
                    ))
                }
                Some(TypeCode::TUPLE_PAIR2) => {
                    let stype = stype()?;
                    STuple(stuple::STuple::pair(SType::sigma_parse(r)?, stype))
                }
                Some(TypeCode::TUPLE_PAIR_SYMMETRIC) if embeddable.is_none() => {
                    STuple(stuple::STuple::quadruple(
                        stype()?,
                        SType::sigma_parse(r)?,
                        SType::sigma_parse(r)?,
                        SType::sigma_parse(r)?,
                    ))
                }
                Some(TypeCode::TUPLE_PAIR_SYMMETRIC) => {
                    let stype = stype()?;
                    STuple(stuple::STuple::pair(stype.clone(), stype))
                }
                #[allow(clippy::unreachable)] // All possible container types are checked
                _ => unreachable!(),
            })
        } else {
            Ok(match TypeCode::parse(c)? {
                TypeCode::TUPLE => {
                    let len = r.get_u8()?;
                    let mut items = Vec::with_capacity(len as usize);
                    for _ in 0..len {
                        items.push(SType::sigma_parse(r)?);
                    }
                    STuple(stuple::STuple::try_from(items)?)
                }
                TypeCode::SANY => SAny,
                TypeCode::SUNIT => SUnit,
                TypeCode::SBOX => SBox,
                TypeCode::SAVL_TREE => SAvlTree,
                TypeCode::SCONTEXT => SContext,
                TypeCode::SSTRING => SString,
                TypeCode::STYPE_VAR => {
                    STypeVar(crate::types::stype_param::STypeVar::sigma_parse(r)?)
                }
                TypeCode::SHEADER => SHeader,
                TypeCode::SPRE_HEADER => SPreHeader,
                TypeCode::SGLOBAL => SGlobal,
                TypeCode::SFUNC if r.tree_version() >= ErgoTreeVersion::V3 => {
                    let t_dom_len = r.get_u8()?;
                    let t_dom = (0..t_dom_len)
                        .map(|_| SType::sigma_parse(r))
                        .collect::<Result<Vec<_>, _>>()?;
                    let t_range = SType::sigma_parse(r)?;
                    let tpe_params_len = r.get_u8()?;
                    let mut tpe_params = vec![];
                    for _ in 0..tpe_params_len {
                        let tpe = SType::sigma_parse(r)?;
                        if let SType::STypeVar(typevar) = tpe {
                            tpe_params.push(STypeParam { ident: typevar });
                        } else {
                            return Err(SigmaParsingError::Misc(
                                "SFunc.tpe_params: only STypeVar is allowed".into(),
                            ));
                        }
                    }
                    SType::SFunc(crate::types::sfunc::SFunc {
                        t_dom,
                        t_range: t_range.into(),
                        tpe_params,
                    })
                }
                #[allow(clippy::unreachable)] // All types with typecode >= Tuple are checked
                _ => unreachable!(),
            })
        }
    }
}

/// Each SType is serialized to array of bytes by:
/// - emitting typeCode of each node (see special case for collections below)
/// - then recursively serializing subtrees from left to right on each level
/// - for each collection of primitive type there is special type code to emit single byte instead of two bytes
///
/// Types code intervals
/// - (1 .. MaxPrimTypeCode)  // primitive types
/// - (CollectionTypeCode .. CollectionTypeCode + MaxPrimTypeCode) // collections of primitive types
/// - (MaxCollectionTypeCode ..)  // Other types
///
/// Collection of non-primitive type is serialized as (CollectionTypeCode, serialize(elementType))
impl SigmaSerializable for SType {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> SigmaSerializeResult {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L25-L25
        use SType::*;
        match self {
            #[allow(clippy::unwrap_used)]
            // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
            stype if stype.is_prim() => TypeCode::from_primitive_type(stype)
                .unwrap()
                .sigma_serialize(w),
            SOption(elem_type) => match &**elem_type {
                #[allow(clippy::unwrap_used)]
                // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement | SSigmaProp
                | SUnsignedBigInt => w
                    .put_u8(
                        (TypeCode::OPTION as u8)
                            + (TypeCode::from_primitive_type(elem_type).unwrap() as u8),
                    )
                    .map_err(From::from),
                SColl(inner_elem_type) => match &**inner_elem_type {
                    #[allow(clippy::unwrap_used)]
                    // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                    SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                    | SSigmaProp | SUnsignedBigInt => w
                        .put_u8(
                            (TypeCode::OPTION_COLL as u8)
                                + TypeCode::from_primitive_type(inner_elem_type).unwrap() as u8,
                        )
                        .map_err(From::from),
                    STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                    | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                    | SGlobal => {
                        // if not "embeddable" type fallback to generic Option type code following
                        // elem type code
                        TypeCode::OPTION.sigma_serialize(w)?;
                        elem_type.sigma_serialize(w)
                    }
                },
                STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | STuple(_)
                | SFunc(_) | SContext | SString | SHeader | SPreHeader | SGlobal => {
                    // if not "embeddable" type fallback to generic Option type code following
                    // elem type code
                    TypeCode::OPTION.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },

            SType::SColl(elem_type) => match &**elem_type {
                #[allow(clippy::unwrap_used)]
                // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement | SSigmaProp
                | SUnsignedBigInt => {
                    w.put_u8(
                        TypeCode::COLL as u8
                            + TypeCode::from_primitive_type(elem_type).unwrap() as u8,
                    )?;
                    // fast-path combined type code = one byte; meter it like TypeCode::sigma_serialize
                    w.add_put_byte_cost();
                    Ok(())
                }
                SColl(inner_elem_type) => match &**inner_elem_type {
                    #[allow(clippy::unwrap_used)]
                    // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                    SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                    | SSigmaProp | SUnsignedBigInt => {
                        w.put_u8(
                            TypeCode::NESTED_COLL as u8
                                + TypeCode::from_primitive_type(inner_elem_type).unwrap() as u8,
                        )?;
                        w.add_put_byte_cost();
                        Ok(())
                    }
                    STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                    | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                    | SGlobal => {
                        // if not "embeddable" type fallback to generic Coll type code following
                        // elem type code
                        TypeCode::COLL.sigma_serialize(w)?;
                        elem_type.sigma_serialize(w)
                    }
                },
                STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | STuple(_)
                | SFunc(_) | SContext | SString | SHeader | SPreHeader | SGlobal => {
                    // if not "embeddable" type fallback to generic Coll type code following
                    // elem type code
                    TypeCode::COLL.sigma_serialize(w)?;
                    elem_type.sigma_serialize(w)
                }
            },
            SType::STuple(stuple::STuple { items }) => match items.as_slice() {
                #[allow(clippy::unwrap_used)]
                // TypeCode::from_primitive_type can't fail since it's only called on primitive types here
                [t1, t2] => match (t1, t2) {
                    (
                        SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                        | SSigmaProp | SUnsignedBigInt,
                        t2,
                    ) if t1 == t2 => {
                        w.put_u8(
                            TypeCode::TUPLE_PAIR_SYMMETRIC as u8
                                + TypeCode::from_primitive_type(t1).unwrap() as u8,
                        )?;
                        w.add_put_byte_cost();
                        Ok(())
                    }
                    (
                        SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                        | SSigmaProp | SUnsignedBigInt,
                        t2,
                    ) => {
                        w.put_u8(
                            TypeCode::TUPLE_PAIR1 as u8
                                + TypeCode::from_primitive_type(t1).unwrap() as u8,
                        )?;
                        w.add_put_byte_cost();
                        t2.sigma_serialize(w)
                    }
                    (
                        t1,
                        SBoolean | SByte | SShort | SInt | SLong | SBigInt | SGroupElement
                        | SSigmaProp | SUnsignedBigInt,
                    ) => {
                        w.put_u8(
                            TypeCode::TUPLE_PAIR2 as u8
                                + TypeCode::from_primitive_type(t2).unwrap() as u8,
                        )?;
                        w.add_put_byte_cost();
                        t1.sigma_serialize(w)
                    }
                    (
                        STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                        | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                        | SGlobal,
                        STypeVar(_) | SAny | SUnit | SBox | SAvlTree | SOption(_) | SColl(_)
                        | STuple(_) | SFunc(_) | SContext | SString | SHeader | SPreHeader
                        | SGlobal,
                    ) => {
                        // Pair of non-primitive types (`(SBox, SAvlTree)`, `((Int, Byte), (Boolean,Box))`, etc.)
                        TypeCode::TUPLE_PAIR1.sigma_serialize(w)?;
                        t1.sigma_serialize(w)?;
                        t2.sigma_serialize(w)
                    }
                },
                [t1, t2, t3] => {
                    TypeCode::TUPLE_PAIR2.sigma_serialize(w)?;
                    t1.sigma_serialize(w)?;
                    t2.sigma_serialize(w)?;
                    t3.sigma_serialize(w)
                }
                [t1, t2, t3, t4] => {
                    TypeCode::TUPLE_PAIR_SYMMETRIC.sigma_serialize(w)?;
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
            SType::SFunc(sfunc) if w.tree_version() >= ErgoTreeVersion::V3 => {
                TypeCode::SFUNC.sigma_serialize(w)?;
                w.put_u8(sfunc.t_dom.len().try_into().map_err(|_| {
                    SigmaSerializationError::NotSupported("t_dom.len() must be <= 255".into())
                })?)?;
                sfunc
                    .t_dom
                    .iter()
                    .try_for_each(|arg| arg.sigma_serialize(w))?;
                sfunc.t_range.sigma_serialize(w)?;
                w.put_u8(sfunc.tpe_params.len().try_into().map_err(|_| {
                    SigmaSerializationError::NotSupported("tpe_params.len() must be <= 255".into())
                })?)?;
                sfunc.tpe_params.iter().try_for_each(|tpe_param| {
                    SType::STypeVar(tpe_param.ident.clone()).sigma_serialize(w)
                })
            }
            SType::SFunc(_) => Err(SigmaSerializationError::NotSupported(
                "SFunc serialization is not supported".into(),
            )),
            #[allow(clippy::unreachable)] // Primitive types are covered by if .is_prim() branch
            _ => unreachable!(),
        }
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        // for reference see http://github.com/ScorexFoundation/sigmastate-interpreter/blob/25251c1313b0131835f92099f02cef8a5d932b5e/sigmastate/src/main/scala/sigmastate/serialization/TypeSerializer.scala#L118-L118
        let c = r.get_u8()?;
        Self::parse_with_tag(r, c)
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::serialization::{
        roundtrip_new_feature, sigma_serialize_roundtrip, sigma_serialize_roundtrip_versioned,
    };
    use crate::types::sfunc::SFunc;
    use crate::types::stype_param::STypeVar;

    use proptest::prelude::*;
    #[test]
    fn serialize_invalid_sfunc() {
        // t_dom > 255
        let mut sfunc = SFunc {
            t_dom: vec![SType::SByte; 256],
            t_range: SType::SByte.into(),
            tpe_params: vec![],
        };
        assert!(sigma_serialize_roundtrip_versioned(
            &SType::SFunc(sfunc.clone()),
            ErgoTreeVersion::V3
        )
        .is_err());
        // t_dom length is now in bounds
        sfunc.t_dom.pop();
        roundtrip_new_feature(&SType::SFunc(sfunc.clone()), ErgoTreeVersion::V3);
        // invalid tpe_params size
        sfunc.tpe_params = vec![
            STypeParam {
                ident: STypeVar::t()
            };
            256
        ];
        assert!(
            sigma_serialize_roundtrip_versioned(&SType::SFunc(sfunc), ErgoTreeVersion::V3).is_err()
        );
    }

    /// Each combined fast-path type code (Coll/nested-Coll/tuple-pair of primitives) is a single
    /// byte that must be metered like `TypeCode::sigma_serialize` (PutByteCost = 1). Matches the
    /// blessed v6 `Global.serialize[Box]` register-type entries (Coll[Byte] 178 / Coll[Int] 152 /
    /// Coll[Coll[Byte]] 151 / (Int,Int) 146 / (Int,Long) 147 / (Coll[Byte],Int) 154), each undercharged by 1.
    #[test]
    fn serialize_charges_fastpath_typecode_byte() {
        use crate::serialization::sigma_byte_writer::SigmaByteWriter;
        fn type_put_cost(t: &SType) -> u64 {
            let mut buf = Vec::new();
            let mut w = SigmaByteWriter::new(&mut buf, None);
            w.enable_serialize_cost_tracking();
            t.sigma_serialize(&mut w).unwrap();
            w.serialize_cost()
        }
        let coll = |t: SType| SType::SColl(t.into());
        let pair = |a: SType, b: SType| SType::STuple(stuple::STuple::pair(a, b));
        assert_eq!(type_put_cost(&coll(SType::SByte)), 1); // COLL+prim
        assert_eq!(type_put_cost(&coll(coll(SType::SByte))), 1); // NESTED_COLL+prim
        assert_eq!(type_put_cost(&pair(SType::SInt, SType::SInt)), 1); // TUPLE_PAIR_SYMMETRIC
        assert_eq!(type_put_cost(&pair(SType::SInt, SType::SLong)), 2); // TUPLE_PAIR1 + SLong prim byte
        assert_eq!(type_put_cost(&pair(coll(SType::SByte), SType::SInt)), 2); // TUPLE_PAIR2 + Coll[Byte] byte (blessed (Coll[Byte],Int) box = 154)
    }

    proptest! {
        #[test]
        fn ser_roundtrip(v in any::<SType>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
        #[test]
        fn sfunc_roundtrip(dom in proptest::collection::vec(any::<SType>(), 64), range in any::<SType>(), tpe_params in proptest::collection::vec(proptest::string::string_regex(".{5}").unwrap(), 32)) {
            let tpe_params: Vec<_> = tpe_params
                .into_iter()
                .map(Into::into)
                .map(|b| STypeVar::new_from_bytes(b).unwrap())
                .map(|ident| STypeParam { ident }).collect();
            let sfunc = SFunc {
                t_dom: dom,
                t_range: range.into(),
                tpe_params
            };
            roundtrip_new_feature(&SType::SFunc(sfunc), ErgoTreeVersion::V3);
        }
    }
}

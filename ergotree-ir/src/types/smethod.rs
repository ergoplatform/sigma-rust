use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::types::TypeCode;
use crate::serialization::SigmaParsingError;
use crate::serialization::SigmaSerializable;
use std::collections::HashMap;
use std::io::Error;

use super::sfunc::SFunc;
use super::stype::SType;
use super::stype_companion::STypeCompanion;
use super::stype_param::STypeVar;
use super::type_unify::unify_many;
use super::type_unify::TypeUnificationError;
use crate::serialization::SigmaParsingError::UnknownMethodId;

/// Method id unique among the methods of the same object
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MethodId(pub u8);

impl SigmaSerializable for MethodId {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), Error> {
        w.put_u8(self.0)
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SigmaParsingError> {
        Ok(Self(r.get_u8()?))
    }
}

/// Object method signature
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct SMethod {
    /// Object type companion
    pub obj_type: &'static STypeCompanion,
    method_raw: SMethodDesc,
}

impl SMethod {
    /// Create new SMethod
    pub(crate) const fn new(obj_type: &'static STypeCompanion, method_raw: SMethodDesc) -> SMethod {
        SMethod {
            obj_type,
            method_raw,
        }
    }

    /// Get method from type and method ids
    pub fn from_ids(type_id: TypeCode, method_id: MethodId) -> Result<Self, SigmaParsingError> {
        let obj_type = STypeCompanion::type_by_id(type_id);
        match obj_type.method_by_id(&method_id) {
            Some(m) => Ok(m),
            None => Err(UnknownMethodId(method_id, type_id)),
        }
    }

    /// Type
    pub fn tpe(&self) -> &SFunc {
        &self.method_raw.tpe
    }

    /// Returns method name
    pub fn name(&self) -> &'static str {
        self.method_raw.name
    }

    /// Returns method id
    pub fn method_id(&self) -> MethodId {
        self.method_raw.method_id.clone()
    }

    /// Return new SMethod with type variables substituted
    pub fn with_concrete_types(self, subst: &HashMap<STypeVar, SType>) -> Self {
        let new_tpe = self.method_raw.tpe.clone().with_subst(subst);
        Self {
            method_raw: self.method_raw.with_tpe(new_tpe),
            ..self
        }
    }

    /// Specializes this instance by creating a new [`SMethod`] instance where signature
    /// is specialized with respect to the given object and args types.
    pub fn specialize_for(
        self,
        obj_tpe: SType,
        args: Vec<SType>,
    ) -> Result<SMethod, TypeUnificationError> {
        let mut items2 = vec![obj_tpe];
        let mut args = args;
        items2.append(args.as_mut());
        unify_many(self.tpe().t_dom.clone(), items2).map(|subst| self.with_concrete_types(&subst))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct SMethodDesc {
    pub(crate) name: &'static str,
    pub(crate) method_id: MethodId,
    pub(crate) tpe: SFunc,
}

impl SMethodDesc {
    pub(crate) fn property(
        obj_tpe: SType,
        name: &'static str,
        res_tpe: SType,
        id: MethodId,
    ) -> SMethodDesc {
        SMethodDesc {
            method_id: id,
            name,
            tpe: SFunc {
                t_dom: vec![obj_tpe],
                t_range: res_tpe.into(),
                tpe_params: vec![],
            },
        }
    }
    pub(crate) fn as_method(&self, obj_type: &'static STypeCompanion) -> SMethod {
        SMethod {
            obj_type,
            method_raw: self.clone(),
        }
    }

    pub(crate) fn with_tpe(self, tpe: SFunc) -> Self {
        Self { tpe, ..self }
    }
}

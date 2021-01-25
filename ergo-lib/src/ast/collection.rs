use crate::serialization::op_code::OpCode;
use crate::serialization::sigma_byte_reader::SigmaByteRead;
use crate::serialization::sigma_byte_writer::SigmaByteWrite;
use crate::serialization::SerializationError;
use crate::serialization::SigmaSerializable;
use crate::types::stype::SType;

use super::constant::Constant;
use super::expr::Expr;
use super::expr::InvalidArgumentError;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    elem_tpe: SType,
    items: Vec<Expr>,
    is_bool_const_coll: bool,
}

impl Collection {
    pub fn new(elem_tpe: SType, items: Vec<Expr>) -> Result<Self, InvalidArgumentError> {
        if items.iter().all(|i| i.tpe() == SType::SBoolean) {
            let is_bool_const_coll = elem_tpe == SType::SBoolean
                && items.iter().all(|i| {
                    matches!(
                        i,
                        Expr::Const(Constant {
                            tpe: SType::SBoolean,
                            v: _,
                        })
                    )
                });
            Ok(Collection {
                elem_tpe,
                items,
                is_bool_const_coll,
            })
        } else {
            Err(InvalidArgumentError(format!(
                "expected items to be of the same type {0:?}, got {1:?}",
                elem_tpe, items
            )))
        }
    }

    pub fn tpe(&self) -> SType {
        SType::SColl(self.elem_tpe.clone().into())
    }

    pub fn op_code(&self) -> OpCode {
        if self.is_bool_const_coll {
            OpCode::COLL_DECL_BOOL_CONST
        } else {
            OpCode::COLL_DECL
        }
    }
}

impl SigmaSerializable for Collection {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), std::io::Error> {
        // TODO: handle bool const array
        w.put_u16(self.items.len() as u16)?;
        self.elem_tpe.sigma_serialize(w)?;
        self.items.iter().try_for_each(|i| i.sigma_serialize(w))
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let items_count = r.get_u16()?;
        let elem_tpe = SType::sigma_parse(r)?;
        let mut items = Vec::with_capacity(items_count as usize);
        for _ in 0..items_count {
            items.push(Expr::sigma_parse(r)?);
        }
        Ok(Self::new(elem_tpe, items)?)
    }
}

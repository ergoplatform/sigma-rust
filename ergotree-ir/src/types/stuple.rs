use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

use bounded_vec::BoundedVec;
use bounded_vec::BoundedVecOutOfBounds;

use super::stype::SType;
use super::stype_param::STypeVar;

/// Tuple items with bounds check (2..=255)
pub type TupleItems<T> = BoundedVec<T, 2, 255>;

impl TryFrom<Vec<SType>> for STuple {
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<SType>) -> Result<Self, Self::Error> {
        Ok(STuple {
            items: value.try_into()?,
        })
    }
}

/// Tuple type
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct STuple {
    /// Tuple element types
    pub items: TupleItems<SType>,
}

impl STuple {
    /// Create a tuple type for a given type pair
    pub fn pair(t1: SType, t2: SType) -> Self {
        STuple {
            items: [t1, t2].into(),
        }
    }

    pub(crate) fn with_subst(self, subst: &HashMap<STypeVar, SType>) -> Self {
        STuple {
            items: self.items.mapped(|a| a.with_subst(subst)),
        }
    }
}

use core::convert::TryFrom;
use core::convert::TryInto;
use hashbrown::HashMap;

use alloc::vec;
use alloc::vec::Vec;
use bounded_vec::BoundedVec;
use bounded_vec::BoundedVecOutOfBounds;

use super::stype::SType;
use super::stype_param::STypeVar;

/// Tuple items with bounds check (1..=255). Sigma-state parses a one-item tuple in
/// types, constants and `Tuple` expressions and rejects it only at evaluation.
pub type TupleItems<T> = BoundedVec<T, 1, 255>;

/// Tuple items of a pair
pub fn pair_items<T>(t1: T, t2: T) -> TupleItems<T> {
    #[allow(clippy::unwrap_used)]
    vec![t1, t2].try_into().unwrap()
}

impl TryFrom<Vec<SType>> for STuple {
    type Error = BoundedVecOutOfBounds;

    fn try_from(value: Vec<SType>) -> Result<Self, Self::Error> {
        Ok(STuple {
            items: value.try_into()?,
        })
    }
}

/// Tuple type
#[derive(PartialEq, Eq, Clone)]
pub struct STuple {
    /// Tuple element types
    pub items: TupleItems<SType>,
}

impl core::fmt::Debug for STuple {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.items.clone().to_vec().fmt(f)
    }
}

impl core::fmt::Display for STuple {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "(")?;
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            item.fmt(f)?;
        }
        write!(f, ")")
    }
}

impl STuple {
    /// Create a tuple type for a given type pair
    pub fn pair(t1: SType, t2: SType) -> Self {
        STuple {
            items: pair_items(t1, t2),
        }
    }

    /// Create a triple type
    pub fn triple(t1: SType, t2: SType, t3: SType) -> Self {
        #[allow(clippy::unwrap_used)]
        STuple {
            items: vec![t1, t2, t3].try_into().unwrap(),
        }
    }

    /// Create a quadruple type
    pub fn quadruple(t1: SType, t2: SType, t3: SType, t4: SType) -> Self {
        #[allow(clippy::unwrap_used)]
        STuple {
            items: vec![t1, t2, t3, t4].try_into().unwrap(),
        }
    }

    pub(crate) fn with_subst(&self, subst: &HashMap<STypeVar, SType>) -> Self {
        STuple {
            items: self.items.clone().mapped(|a| a.with_subst(subst)),
        }
    }
}

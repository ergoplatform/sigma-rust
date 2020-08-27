//! ContextExtension type
use crate::{
    ast::Constant,
    serialization::{
        sigma_byte_reader::SigmaByteRead, sigma_byte_writer::SigmaByteWrite, SerializationError,
        SigmaSerializable,
    },
};
use indexmap::IndexMap;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{convert::TryFrom, io, num::ParseIntError};

/// User-defined variables to be put into context
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    feature = "with-serde",
    derive(Serialize, Deserialize),
    serde(
        into = "HashMap<String, Constant>",
        try_from = "HashMap<String, Constant>"
    )
)]
pub struct ContextExtension {
    /// key-value pairs of variable id and it's value
    pub values: IndexMap<u8, Constant>,
}

impl ContextExtension {
    /// Returns an empty ContextExtension
    pub fn empty() -> Self {
        Self {
            values: IndexMap::new(),
        }
    }
}

impl SigmaSerializable for ContextExtension {
    fn sigma_serialize<W: SigmaByteWrite>(&self, w: &mut W) -> Result<(), io::Error> {
        w.put_u8(self.values.len() as u8)?;
        let mut sorted_values: Vec<(&u8, &Constant)> = self.values.iter().collect();
        // stable order is important for tx id generation
        // since JSON encoding does not preserve the order, JSON roundtrip would result in different order
        // of values and thus a different tx id
        // see https://github.com/ScorexFoundation/sigmastate-interpreter/issues/681
        sorted_values.sort_by_key(|(k, _)| *k);
        sorted_values.iter().try_for_each(|(idx, c)| {
            w.put_u8(**idx)?;
            c.sigma_serialize(w)
        })?;
        Ok(())
    }

    fn sigma_parse<R: SigmaByteRead>(r: &mut R) -> Result<Self, SerializationError> {
        let values_count = r.get_u8()?;
        let mut values: IndexMap<u8, Constant> = IndexMap::with_capacity(values_count as usize);
        for _ in 0..values_count {
            let idx = r.get_u8()?;
            values.insert(idx, Constant::sigma_parse(r)?);
        }
        Ok(ContextExtension { values })
    }
}

impl Into<HashMap<String, Constant>> for ContextExtension {
    fn into(self) -> HashMap<String, Constant> {
        self.values
            .into_iter()
            .map(|(k, v)| (format!("{}", k), v))
            .collect()
    }
}

impl TryFrom<HashMap<String, Constant>> for ContextExtension {
    type Error = ParseIntError;
    fn try_from(values_str: HashMap<String, Constant>) -> Result<Self, Self::Error> {
        let values = values_str.iter().try_fold(
            IndexMap::with_capacity(values_str.len()),
            |mut acc, pair| {
                let idx: u8 = pair.0.parse()?;
                acc.insert(idx, pair.1.clone());
                Ok(acc)
            },
        )?;
        Ok(ContextExtension { values })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::sigma_serialize_roundtrip;
    use proptest::{arbitrary::Arbitrary, collection::vec, prelude::*};

    impl Arbitrary for ContextExtension {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<Constant>(), 0..10)
                .prop_map(|constants| {
                    let pairs: Vec<(u8, Constant)> = constants
                        .into_iter()
                        .enumerate()
                        .map(|(idx, c)| (idx as u8, c))
                        .collect();
                    Self {
                        values: pairs.into_iter().collect(),
                    }
                })
                .boxed()
        }
    }

    proptest! {

        #[test]
        fn ser_roundtrip(v in any::<ContextExtension>()) {
            prop_assert_eq![sigma_serialize_roundtrip(&v), v];
        }
    }
}
